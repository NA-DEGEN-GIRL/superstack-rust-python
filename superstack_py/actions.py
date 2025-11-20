from __future__ import annotations

from typing import List, Optional, Union, Literal, Annotated, Any, Dict

from pydantic import BaseModel, Field, AliasChoices, model_serializer, model_validator


# 주의: Rust serde와 동일한 JSON을 내보내기 위해, 필드 alias와 직렬화 규칙을 세밀하게 지정합니다.

class BulkOrder(BaseModel):
    # serde(rename_all = "camelCase")
    orders: List["OrderRequest"]
    grouping: str

    model_config = dict(populate_by_name=True)


class BulkCancel(BaseModel):
    cancels: List["CancelRequest"]

    model_config = dict(populate_by_name=True)


class BulkCancelCloid(BaseModel):
    cancels: List["CancelRequestCloid"]

    model_config = dict(populate_by_name=True)


class ScheduleCancel(BaseModel):
    time: Optional[int] = None  # u64

    model_config = dict(populate_by_name=True)


class BulkModify(BaseModel):
    modifies: List["ModifyRequest"]

    model_config = dict(populate_by_name=True)


class UpdateLeverage(BaseModel):
    asset: int
    is_cross: bool = Field(serialization_alias="isCross", validation_alias=AliasChoices("isCross", "is_cross"))
    leverage: int

    @model_serializer
    def ser(self) -> dict:
        # camelCase 강제 직렬화
        return {
            "asset": self.asset,
            "isCross": self.is_cross,
            "leverage": self.leverage,
        }

    model_config = dict(populate_by_name=True)


class UpdateIsolatedMargin(BaseModel):
    asset: int
    is_buy: bool = Field(serialization_alias="isBuy", validation_alias=AliasChoices("isBuy", "is_buy"))
    ntli: int  # i64

    @model_serializer
    def ser(self) -> dict:
        return {
            "asset": self.asset,
            "isBuy": self.is_buy,
            "ntli": self.ntli,
        }

    model_config = dict(populate_by_name=True)


class UsdSend(BaseModel):
    destination: str
    amount: str

    model_config = dict(populate_by_name=True)


class SpotSend(BaseModel):
    destination: str
    token: str
    amount: str

    model_config = dict(populate_by_name=True)


class UsdClassTransfer(BaseModel):
    amount: str
    to_perp: bool = Field(serialization_alias="toPerp", validation_alias=AliasChoices("toPerp", "to_perp"))

    @model_serializer
    def ser(self) -> dict:
        return {
            "amount": self.amount,
            "toPerp": self.to_perp,
        }

    model_config = dict(populate_by_name=True)


class Limit(BaseModel):
    # struct Limit { tif: String }
    tif: str

    model_config = dict(populate_by_name=True)


class Trigger(BaseModel):
    # serde(rename_all = "camelCase")
    is_market: bool = Field(serialization_alias="isMarket", validation_alias=AliasChoices("isMarket", "is_market"))
    trigger_px: str = Field(serialization_alias="triggerPx", validation_alias=AliasChoices("triggerPx", "trigger_px"))
    tpsl: str

    @model_serializer
    def ser(self) -> dict:
        return {
            "isMarket": self.is_market,
            "triggerPx": self.trigger_px,
            "tpsl": self.tpsl,
        }

    model_config = dict(populate_by_name=True)


class Order(BaseModel):
    """
    Rust: enum Order { Limit(Limit), Trigger(Trigger) } + rename_all = "camelCase"
    serde 기본(Externally Tagged)과 동일한 형태:
      {"limit": {...}} 또는 {"trigger": {...}}
    """
    limit: Optional[Limit] = None
    trigger: Optional[Trigger] = None

    @model_validator(mode="after")
    def _check_exactly_one(self):
        count = int(self.limit is not None) + int(self.trigger is not None)
        if count != 1:
            raise ValueError("Order must have exactly one of {'limit','trigger'} set.")
        return self

    @classmethod
    def limit_order(cls, *, tif: str) -> "Order":
        return cls(limit=Limit(tif=tif))

    @classmethod
    def trigger_order(cls, *, is_market: bool, trigger_px: str, tpsl: str) -> "Order":
        return cls(trigger=Trigger(is_market=is_market, trigger_px=trigger_px, tpsl=tpsl))

    @model_serializer(mode="wrap")
    def ser(self, handler):
        if self.limit is not None:
            return {"limit": self.limit.model_dump(by_alias=True)}
        else:
            return {"trigger": self.trigger.model_dump(by_alias=True)}

    @model_validator(mode="before")
    @classmethod
    def parse_externally_tagged(cls, value):
        # 입력이 {"limit": {...}} 또는 {"trigger": {...}}인 경우 파싱
        if isinstance(value, dict):
            if "limit" in value:
                return {"limit": value["limit"], "trigger": None}
            if "trigger" in value:
                return {"limit": None, "trigger": value["trigger"]}
        return value

    model_config = dict(populate_by_name=True)


class OrderRequest(BaseModel):
    """
    Rust:
      #[serde(rename = "a", alias="asset")] pub asset: u32,
      #[serde(rename = "b", alias="isBuy")] pub is_buy: bool,
      #[serde(rename = "p", alias="limitPx")] pub limit_px: String,
      #[serde(rename = "s", alias="sz")]     pub sz: String,
      #[serde(rename = "r", alias="reduceOnly", default)] pub reduce_only: bool,
      #[serde(rename = "t", alias="orderType")] pub order_type: Order,
      #[serde(rename = "c", alias="cloid", skip_serializing_if=Option::is_none)] pub cloid: Option<String>,
    - 직렬화 시 단문 키 a/b/p/s/r/t/c 로 나가야 하므로 serialization_alias를 사용합니다.
    - 역직렬화 시 a 또는 긴 alias 모두 허용하려고 validation_alias=AliasChoices 사용합니다.
    """
    asset: int = Field(
        validation_alias=AliasChoices("a", "asset"),
        serialization_alias="a",
    )
    is_buy: bool = Field(
        validation_alias=AliasChoices("b", "isBuy", "is_buy"),
        serialization_alias="b",
    )
    limit_px: str = Field(
        validation_alias=AliasChoices("p", "limitPx", "limit_px"),
        serialization_alias="p",
    )
    sz: str = Field(
        validation_alias=AliasChoices("s", "sz"),
        serialization_alias="s",
    )
    reduce_only: bool = Field(
        default=False,
        validation_alias=AliasChoices("r", "reduceOnly", "reduce_only"),
        serialization_alias="r",
    )
    order_type: Order = Field(
        validation_alias=AliasChoices("t", "orderType", "order_type"),
        serialization_alias="t",
    )
    cloid: Optional[str] = Field(
        default=None,
        validation_alias=AliasChoices("c", "cloid"),
        serialization_alias="c",
    )

    model_config = dict(populate_by_name=True)


class CancelRequest(BaseModel):
    asset: int = Field(validation_alias=AliasChoices("a", "asset"), serialization_alias="a")
    oid: int = Field(validation_alias=AliasChoices("o", "oid"), serialization_alias="o")

    model_config = dict(populate_by_name=True)


class CancelRequestCloid(BaseModel):
    asset: int
    cloid: str

    model_config = dict(populate_by_name=True)


class ModifyRequest(BaseModel):
    oid: int
    order: OrderRequest

    model_config = dict(populate_by_name=True)


# Actions (serde(tag="type"), rename_all="camelCase")
class _OrderAction(BaseModel):
    type: Literal["order"] = "order"
    orders: List[OrderRequest]
    grouping: str

class _CancelAction(BaseModel):
    type: Literal["cancel"] = "cancel"
    cancels: List[CancelRequest]

class _CancelByCloidAction(BaseModel):
    type: Literal["cancelByCloid"] = "cancelByCloid"
    cancels: List[CancelRequestCloid]

class _ScheduleCancelAction(BaseModel):
    type: Literal["scheduleCancel"] = "scheduleCancel"
    time: Optional[int] = None

class _BatchModifyAction(BaseModel):
    type: Literal["batchModify"] = "batchModify"
    modifies: List[ModifyRequest]

class _UpdateLeverageAction(BaseModel):
    type: Literal["updateLeverage"] = "updateLeverage"
    asset: int
    is_cross: bool = Field(serialization_alias="isCross", validation_alias=AliasChoices("isCross", "is_cross"))
    leverage: int

    @model_serializer
    def ser(self) -> dict:
        return {
            "type": "updateLeverage",
            "asset": self.asset,
            "isCross": self.is_cross,
            "leverage": self.leverage,
        }

class _UpdateIsolatedMarginAction(BaseModel):
    type: Literal["updateIsolatedMargin"] = "updateIsolatedMargin"
    asset: int
    is_buy: bool = Field(serialization_alias="isBuy", validation_alias=AliasChoices("isBuy", "is_buy"))
    ntli: int

    @model_serializer
    def ser(self) -> dict:
        return {
            "type": "updateIsolatedMargin",
            "asset": self.asset,
            "isBuy": self.is_buy,
            "ntli": self.ntli,
        }

class _UsdSendAction(BaseModel):
    type: Literal["usdSend"] = "usdSend"
    destination: str
    amount: str

class _SpotSendAction(BaseModel):
    type: Literal["spotSend"] = "spotSend"
    destination: str
    token: str
    amount: str

class _UsdClassTransferAction(BaseModel):
    type: Literal["usdClassTransfer"] = "usdClassTransfer"
    amount: str
    to_perp: bool = Field(serialization_alias="toPerp", validation_alias=AliasChoices("toPerp", "to_perp"))

    @model_serializer
    def ser(self) -> dict:
        return {
            "type": "usdClassTransfer",
            "amount": self.amount,
            "toPerp": self.to_perp,
        }


Actions = Annotated[
    Union[
        _OrderAction,
        _CancelAction,
        _CancelByCloidAction,
        _ScheduleCancelAction,
        _BatchModifyAction,
        _UpdateLeverageAction,
        _UpdateIsolatedMarginAction,
        _UsdSendAction,
        _SpotSendAction,
        _UsdClassTransferAction,
    ],
    Field(discriminator="type"),
]