from __future__ import annotations

import json
from typing import Optional, Any

import httpx
from pydantic import BaseModel

from .actions import (
    Actions, BulkOrder, BulkCancel, BulkCancelCloid, ScheduleCancel, BulkModify,
    UpdateLeverage, UpdateIsolatedMargin, UsdSend, SpotSend, UsdClassTransfer,
    Order, OrderRequest, ModifyRequest,
)
from .types import (
    ApiInfoResponse, ErrorResponse,
    ExchangePayload, ExchangeRequest, ExchangeResponse,
)
from .hype_responses import HypeExchangeResponseStatus, HypeExchangeResponse


DEFAULT_BASE_URL = "https://wallet-service.superstack.xyz"
WIRE_DECIMALS: int = 8  # Rust: const WIRE_DECIMALS: u8 = 8


def float_to_string_for_hashing(x: float) -> str:
    """
    Rust 동작 동일:
      - 소수점 8자리 고정 포맷
      - 후행 '0' 제거
      - 후행 '.' 제거
      - "-0" -> "0"
    """
    s = f"{x:.{WIRE_DECIMALS}f}"
    # 후행 0 제거
    while s.endswith("0"):
        s = s[:-1]
    # 후행 '.' 제거
    if s.endswith("."):
        s = s[:-1]
    if s == "-0":
        return "0"
    return s


class SuperstackApiClient:
    def __init__(
        self,
        api_key: str,
        base_url: str = DEFAULT_BASE_URL,
        http_client: Optional[httpx.AsyncClient] = None,
    ) -> None:
        self.api_key = api_key
        self.base_url = base_url
        self._own_client = http_client is None
        self.http = http_client or httpx.AsyncClient()

    @classmethod
    def with_base_url(cls, api_key: str, base_url: str) -> "SuperstackApiClient":
        return cls(api_key=api_key, base_url=base_url)

    @classmethod
    def with_base_url_and_http_client(
        cls, api_key: str, base_url: str, http_client: httpx.AsyncClient
    ) -> "SuperstackApiClient":
        return cls(api_key=api_key, base_url=base_url, http_client=http_client)

    async def aclose(self) -> None:
        if self._own_client:
            await self.http.aclose()

    async def __aenter__(self) -> "SuperstackApiClient":
        return self

    async def __aexit__(self, exc_type, exc, tb):
        await self.aclose()

    # ===== API methods =====

    async def get_api_info(self) -> ApiInfoResponse:
        url = f"{self.base_url}/api/info"
        headers = {"Authorization": f"Bearer {self.api_key}"}
        resp = await self.http.get(url, headers=headers)
        if resp.status_code >= 400:
            await self._raise_http_error(resp)
        return ApiInfoResponse.model_validate(resp.json())

    async def _post_hyperliquid_exchange(
        self, exchange_payload: ExchangePayload
    ) -> HypeExchangeResponse:
        url = "https://api.hyperliquid.xyz/exchange"
        headers = {"Content-Type": "application/json"}
        # Rust는 serde_json::to_string(&exchange_payload) 형태 → by_alias 필요
        body = exchange_payload.json_payload
        resp = await self.http.post(url, headers=headers, json=body)
        if resp.status_code >= 400:
            text = await resp.aread()
            raise RuntimeError(
                f"Request to hyperliquid exchange endpoint failed with status "
                f"{resp.status_code}, error: {text.decode(errors='ignore')}"
            )
        # HypeExchangeResponseStatus::Ok(response) | ::Err(error) 처리
        status_obj = HypeExchangeResponseStatus.__pydantic_validator__.validate_python(resp.json())
        if isinstance(status_obj, dict):  # 안전망
            status = status_obj.get("status")
            if status == "Ok":
                return HypeExchangeResponse.model_validate(status_obj["response"])
            raise RuntimeError(status_obj.get("response", "Unknown error"))

        # pydantic 모델 분기
        if status_obj.status == "Ok":
            return status_obj.response
        else:
            raise RuntimeError(status_obj.response)

    async def _post_wallet_api_exchange(
        self,
        action: Actions,
        vault_address: Optional[str],
        expires_after: Optional[int],
    ) -> ExchangeResponse:
        url = f"{self.base_url}/api/exchange"
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        # ExchangeRequest(action, vaultAddress, expiresAfter) - camelCase 직렬화
        # pydantic 모델 → dict(by_alias=True, exclude_none=True)
        # action은 pydantic 모델(Union)이어야 함
        if hasattr(action, "model_dump"):
            action_payload: Any = action.model_dump(by_alias=True, exclude_none=True)
        else:
            action_payload = action  # dict 형태도 허용

        req = ExchangeRequest(
            action=action_payload,
            vault_address=vault_address,
            expires_after=expires_after,
        )
        resp = await self.http.post(url, headers=headers, json=req.json_payload)
        if resp.status_code >= 400:
            await self._raise_http_error(resp)
        return ExchangeResponse.model_validate(resp.json())

    async def _post_exchange(
        self,
        action: Actions,
        vault_address: Optional[str],
        expires_after: Optional[int],
    ) -> HypeExchangeResponse:
        exchange_response = await self._post_wallet_api_exchange(action, vault_address, expires_after)
        # 디버그 시 직렬화된 문자열을 참고하려면 아래 주석을 해제하세요.
        # print("exchange_response:", json.dumps(exchange_response.model_dump(by_alias=True), ensure_ascii=False))
        return await self._post_hyperliquid_exchange(exchange_response.payload)

    # ---- high-level helpers (Rust 동명 메서드) ----

    async def order(
        self,
        bulk_order: BulkOrder,
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        # Rust: 문자열로 들어온 수치를 f64→정규 포맷 문자열로 변환
        for o in bulk_order.orders:
            o.limit_px = float_to_string_for_hashing(float(o.limit_px))
            o.sz = float_to_string_for_hashing(float(o.sz))
            # Order.Trigger인 경우 trigger_px도 정규화
            if o.order_type.trigger is not None:
                o.order_type.trigger.trigger_px = float_to_string_for_hashing(
                    float(o.order_type.trigger.trigger_px)
                )
        action = _order_action_from_bulk(bulk_order)
        return await self._post_exchange(action, vault_address, expires_after)

    async def cancel(
        self,
        cancels: BulkCancel,
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        action = _cancel_action_from_bulk(cancels)
        return await self._post_exchange(action, vault_address, expires_after)

    async def cancel_by_cloid(
        self,
        cancels: BulkCancelCloid,
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        action = _cancel_by_cloid_action_from_bulk(cancels)
        return await self._post_exchange(action, vault_address, expires_after)

    async def schedule_cancel(
        self,
        time: Optional[int],
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        action = _schedule_cancel_action(time)
        return await self._post_exchange(action, vault_address, expires_after)

    async def modify(
        self,
        modifies: BulkModify,
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        for m in modifies.modifies:
            m.order.limit_px = float_to_string_for_hashing(float(m.order.limit_px))
            m.order.sz = float_to_string_for_hashing(float(m.order.sz))
            if m.order.order_type.trigger is not None:
                m.order.order_type.trigger.trigger_px = float_to_string_for_hashing(
                    float(m.order.order_type.trigger.trigger_px)
                )
        action = _batch_modify_action(modifies)
        return await self._post_exchange(action, vault_address, expires_after)

    async def update_leverage(
        self,
        asset: int,
        is_cross: bool,
        leverage: int,
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        action = _update_leverage_action(asset, is_cross, leverage)
        return await self._post_exchange(action, vault_address, expires_after)

    async def update_isolated_margin(
        self,
        asset: int,
        amount: float,
        vault_address: Optional[str] = None,
        expires_after: Optional[int] = None,
    ) -> HypeExchangeResponse:
        # Rust: (amount * 1_000_000.0).round() as i64; is_buy = true
        ntli = int(round(amount * 1_000_000.0))
        action = _update_isolated_margin_action(asset, is_buy=True, ntli=ntli)
        return await self._post_exchange(action, vault_address, expires_after)

    async def usdc_transfer(self, amount: str, destination: str) -> HypeExchangeResponse:
        action = _usd_send_action(amount, destination)
        return await self._post_exchange(action, vault_address=None, expires_after=None)

    async def spot_transfer(self, amount: str, destination: str, token: str) -> HypeExchangeResponse:
        action = _spot_send_action(amount, destination, token)
        return await self._post_exchange(action, vault_address=None, expires_after=None)

    async def usd_class_transfer(self, amount: str, to_perp: bool) -> HypeExchangeResponse:
        action = _usd_class_transfer_action(amount, to_perp)
        return await self._post_exchange(action, vault_address=None, expires_after=None)

    # ===== helpers =====

    async def _raise_http_error(self, resp: httpx.Response) -> None:
        text = resp.text
        try:
            err_obj = ErrorResponse.model_validate_json(text)
            raise RuntimeError(f"Request failed with status {resp.status_code}, error: {err_obj.error}")
        except Exception:
            # JSON 파싱 실패 시, 원문 메시지 동봉
            raise RuntimeError(f"Request failed with status {resp.status_code}, failed to parse error response: {text}")


# Internal action builders to keep type hints simple
def _order_action_from_bulk(bulk: BulkOrder) -> Actions:
    from .actions import _OrderAction
    return _OrderAction(orders=bulk.orders, grouping=bulk.grouping)

def _cancel_action_from_bulk(bulk: BulkCancel) -> Actions:
    from .actions import _CancelAction
    return _CancelAction(cancels=bulk.cancels)

def _cancel_by_cloid_action_from_bulk(bulk: BulkCancelCloid) -> Actions:
    from .actions import _CancelByCloidAction
    return _CancelByCloidAction(cancels=bulk.cancels)

def _schedule_cancel_action(time: Optional[int]) -> Actions:
    from .actions import _ScheduleCancelAction
    return _ScheduleCancelAction(time=time)

def _batch_modify_action(modifies: BulkModify) -> Actions:
    from .actions import _BatchModifyAction
    return _BatchModifyAction(modifies=modifies.modifies)

def _update_leverage_action(asset: int, is_cross: bool, leverage: int) -> Actions:
    from .actions import _UpdateLeverageAction
    return _UpdateLeverageAction(asset=asset, is_cross=is_cross, leverage=leverage)

def _update_isolated_margin_action(asset: int, is_buy: bool, ntli: int) -> Actions:
    from .actions import _UpdateIsolatedMarginAction
    return _UpdateIsolatedMarginAction(asset=asset, is_buy=is_buy, ntli=ntli)

def _usd_send_action(amount: str, destination: str) -> Actions:
    from .actions import _UsdSendAction
    return _UsdSendAction(amount=amount, destination=destination)

def _spot_send_action(amount: str, destination: str, token: str) -> Actions:
    from .actions import _SpotSendAction
    return _SpotSendAction(amount=amount, destination=destination, token=token)

def _usd_class_transfer_action(amount: str, to_perp: bool) -> Actions:
    from .actions import _UsdClassTransferAction
    return _UsdClassTransferAction(amount=amount, to_perp=to_perp)