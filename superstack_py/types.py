from __future__ import annotations

from typing import Optional, Any, Dict, Union, Literal, Annotated
from enum import Enum  # comment: [추가] Enum 사용

from pydantic import BaseModel, Field, AliasChoices


# 서버에서 내려오는 API Info는 Rust에서 rename_all을 사용하지 않았으므로
# 필드명 그대로(snake_case) 파싱합니다.

class Network(str, Enum):  # comment: [변경] Pydantic이 인식 가능한 Enum으로 선언
    # Rust enum과 동일한 문자열 표현을 유지 ("Ethereum", "Solana")
    Ethereum = "Ethereum"
    Solana = "Solana"


# WalletSet는 Rust의 enum(Unit, Tuple)이므로 실제 응답 형태가
# "Main" 또는 {"Secondary": 2} / {"Forward": 3} 형태가 될 수 있습니다.
# 파싱 관용성을 위해 유연하게 처리합니다.
WalletSet = Annotated[
    Union[
        Literal["Main"],
        Dict[Literal["Secondary"], int],
        Dict[Literal["Forward"], int],
        str,  # 호환성(만약 단순 문자열로 내려오는 경우)
    ],
    Field(),
]


class WalletInfo(BaseModel):
    address: str
    network: Network  # 문자열 → Enum 자동 변환
    wallet_set: WalletSet

    model_config = dict(populate_by_name=True)


class UserInfo(BaseModel):
    id: str  # Uuid 문자열
    wallets: list[WalletInfo]

    def get_evm_main_wallet(self) -> Optional[WalletInfo]:
        for w in self.wallets:
            if w.network == Network.Ethereum and (w.wallet_set == "Main"):
                return w
        return None

    def get_solana_main_wallet(self) -> Optional[WalletInfo]:
        for w in self.wallets:
            if w.network == Network.Solana and (w.wallet_set == "Main"):
                return w
        return None

    model_config = dict(populate_by_name=True)


class ApiInfo(BaseModel):
    id: str          # Uuid
    created_at: int  # Unix timestamp
    ip_whitelist: Optional[list[str]] = None  # 문자열 표현 허용 (예: "1.2.3.4/32")
    permissions: int

    model_config = dict(populate_by_name=True)


class ApiInfoResponse(BaseModel):
    user_info: UserInfo
    api_info: ApiInfo

    model_config = dict(populate_by_name=True)


class ErrorResponse(BaseModel):
    error: str

    model_config = dict(populate_by_name=True)


# 요청/응답 페이로드 (serde(rename_all="camelCase"))
class ExchangeRequest(BaseModel):
    action: Any  # Actions(pydantic 모델)를 그대로 넣으면 dict로 직렬화됨
    vault_address: Optional[str] = Field(default=None, serialization_alias="vaultAddress", validation_alias=AliasChoices("vaultAddress", "vault_address"))
    expires_after: Optional[int] = Field(default=None, serialization_alias="expiresAfter", validation_alias=AliasChoices("expiresAfter", "expires_after"))

    @property
    def json_payload(self) -> dict:
        return self.model_dump(by_alias=True, exclude_none=True)

    model_config = dict(populate_by_name=True)


class ExchangePayload(BaseModel):
    action: Any
    nonce: int
    signature: Any
    vault_address: Optional[str] = Field(default=None, serialization_alias="vaultAddress", validation_alias=AliasChoices("vaultAddress", "vault_address"))
    expires_after: Optional[int] = Field(default=None, serialization_alias="expiresAfter", validation_alias=AliasChoices("expiresAfter", "expires_after"))

    @property
    def json_payload(self) -> dict:
        return self.model_dump(by_alias=True, exclude_none=True)

    model_config = dict(populate_by_name=True)


class ExchangeResponse(BaseModel):
    payload: ExchangePayload

    model_config = dict(populate_by_name=True)