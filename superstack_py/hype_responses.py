from __future__ import annotations

from typing import Optional, Any, Literal, Annotated, Union

from pydantic import BaseModel, Field


# Rust의 상세 enum(HypeExchangeDataStatus 등)을 완전 동일 모델로 파싱하려면
# externally-tagged union을 여럿 구현해야 합니다.
# 실사용 경로(클라이언트가 hyperliquid 응답을 받아 그대로 리턴) 기준으로
# 안전하게 쓰기 위해 상위 구조를 엄밀히 정의하고, 내부 data.statuses는 dict로 수용합니다.

class HypeExchangeResponse(BaseModel):
    response_type: str = Field(alias="type")
    data: Optional[dict] = None  # { "statuses": [...] } 형태이지만, 호환성 위해 dict

    model_config = dict(populate_by_name=True)


class _HypeOk(BaseModel):
    status: Literal["Ok"]
    response: HypeExchangeResponse


class _HypeErr(BaseModel):
    status: Literal["Err"]
    response: str


HypeExchangeResponseStatus = Annotated[
    Union[_HypeOk, _HypeErr],
    Field(discriminator="status"),
]