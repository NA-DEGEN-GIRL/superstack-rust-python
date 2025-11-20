import os
import json
import asyncio
from datetime import datetime, timezone

# .env 파일 사용을 원하면 주석 해제하세요.
from dotenv import load_dotenv
load_dotenv()

from superstack import SuperstackApiClient  # comment: 포팅한 Python SDK 사용

async def main() -> None:
    # comment: Rust 예제와 동일하게 API_KEY를 우선 사용, 없으면 WALLET_API_KEY 사용
    api_key = os.getenv("API_KEY") or os.getenv("WALLET_API_KEY")
    if not api_key:
        raise SystemExit("Set API_KEY or WALLET_API_KEY environment variable.")

    # comment: Rust 기본값과 동일. 필요시 WALLET_API_BASE_URL로 오버라이드
    base_url = os.getenv("WALLET_API_BASE_URL", "https://wallet-service.superstack.xyz")

    client = SuperstackApiClient(api_key=api_key, base_url=base_url)

    try:
        # GET /api/info
        api_info = await client.get_api_info()

        # Pretty JSON 출력
        print("API information:\n", json.dumps(api_info.model_dump(), indent=2))

        # EVM Main Wallet
        evm_main = api_info.user_info.get_evm_main_wallet()
        if evm_main:
            print("EVM Main Wallet:", evm_main.address)
        else:
            print("EVM Main Wallet: <not found>")

        # API Key 정보
        print("API Key ID:", api_info.api_info.id)
        created_at_dt = datetime.fromtimestamp(api_info.api_info.created_at, tz=timezone.utc)
        print("API Key created at:", created_at_dt.strftime("%Y-%m-%d %H:%M:%S UTC"))
        print("API Key IP whitelist:", api_info.api_info.ip_whitelist)
        print("API Key permissions:", api_info.api_info.permissions)
    finally:
        await client.aclose()

if __name__ == "__main__":
    asyncio.run(main())