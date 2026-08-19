import os
import time
import traceback

from dotenv import load_dotenv
from fastapi import FastAPI, HTTPException, Request
from openai import (
    OpenAI,
    APIConnectionError,
    APITimeoutError,
    APIStatusError,
    RateLimitError,
)
import uvicorn


# ============================================================
# Environment
# ============================================================

BASE_DIR = os.path.dirname(
    os.path.abspath(__file__)
)

ENV_FILE = os.path.join(
    BASE_DIR,
    ".env"
)

load_dotenv(ENV_FILE)


# ============================================================
# Configuration
# ============================================================

API_KEY = os.getenv(
    "OPENAI_API_KEY"
)

MODEL = os.getenv(
    "OPENAI_MODEL",
    "gpt-5"
)

HOST = os.getenv(
    "HOST",
    "127.0.0.1"
)

PORT = int(
    os.getenv(
        "PORT",
        "8000"
    )
)

# This is a network/API timeout, not an agent-round limit.
# It prevents a dead HTTP connection from hanging forever.
OPENAI_TIMEOUT = float(
    os.getenv(
        "OPENAI_TIMEOUT",
        "1800"
    )
)

OPENAI_MAX_RETRIES = int(
    os.getenv(
        "OPENAI_MAX_RETRIES",
        "2"
    )
)


if not API_KEY:
    raise RuntimeError(
        f"OPENAI_API_KEY was not found in {ENV_FILE}"
    )


# ============================================================
# OpenAI client
# ============================================================

client = OpenAI(
    api_key=API_KEY,
    timeout=OPENAI_TIMEOUT,
    max_retries=OPENAI_MAX_RETRIES,
)


# ============================================================
# FastAPI
# ============================================================

app = FastAPI(
    title="My AI Coder API",
    version="2.0.0"
)


# ============================================================
# Health check
# ============================================================

@app.get("/")
async def root():

    return {
        "status": "online",
        "message": "My AI Coder API is running",
        "model": MODEL,
    }


@app.get("/health")
async def health():

    return {
        "status": "ok",
        "model": MODEL,
    }


# ============================================================
# Chat
# ============================================================

@app.post("/chat")
async def chat(
    request: Request
):

    request_started = time.perf_counter()

    try:

        body = await request.json()

        # ----------------------------------------------------
        # Validate messages
        # ----------------------------------------------------

        if "messages" in body:

            messages = body["messages"]

            if not isinstance(
                messages,
                list
            ):
                raise HTTPException(
                    status_code=400,
                    detail=(
                        "'messages' must be an array."
                    )
                )

        elif "message" in body:

            user_message = body["message"]

            if not isinstance(
                user_message,
                str
            ):
                raise HTTPException(
                    status_code=400,
                    detail=(
                        "'message' must be a string."
                    )
                )

            messages = [
                {
                    "role": "user",
                    "content": user_message
                }
            ]

        else:

            raise HTTPException(
                status_code=400,
                detail=(
                    "Request must contain "
                    "'message' or 'messages'."
                )
            )

        # ----------------------------------------------------
        # Model
        # ----------------------------------------------------

        model = body.get(
            "model",
            MODEL
        )

        # ----------------------------------------------------
        # Build OpenAI request
        #
        # Deliberately only pass the fields that your extension
        # actually needs.
        # ----------------------------------------------------

        request_data = {
            "model": model,
            "messages": messages,
        }

        if (
            "tools" in body and
            body["tools"] is not None
        ):

            request_data["tools"] = body["tools"]

        if (
            "tool_choice" in body and
            body["tool_choice"] is not None
        ):

            request_data["tool_choice"] = (
                body["tool_choice"]
            )

        # Optional parameters that may be useful.
        #
        # They are passed only when the extension explicitly
        # sends them.

        optional_fields = [
            "temperature",
            "max_tokens",
            "max_completion_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "stop",
        ]

        for field in optional_fields:

            value = body.get(
                field
            )

            if value is not None:

                request_data[field] = value

        # ----------------------------------------------------
        # Logging
        #
        # DO NOT print the complete request body.
        # Your tool calls can contain entire files.
        # ----------------------------------------------------

        tool_count = len(
            body.get(
                "tools",
                []
            )
        )

        message_count = len(
            messages
        )

        last_role = (
            messages[-1].get("role")
            if messages
            else None
        )

        print()
        print(
            "========== CHAT REQUEST =========="
        )
        print(
            f"model={model}"
        )
        print(
            f"messages={message_count}"
        )
        print(
            f"tools={tool_count}"
        )
        print(
            f"last_role={last_role}"
        )
        print(
            "=================================="
        )
        print()

        # ----------------------------------------------------
        # OpenAI
        # ----------------------------------------------------

        print(
            "[server] Calling OpenAI..."
        )

        openai_started = time.perf_counter()

        response = client.chat.completions.create(
            **request_data
        )

        openai_elapsed = (
            time.perf_counter()
            - openai_started
        )

        # ----------------------------------------------------
        # Extract assistant message
        # ----------------------------------------------------

        assistant_message = (
            response.choices[0].message
        )

        message_data = (
            assistant_message.model_dump(
                exclude_none=True
            )
        )

        total_elapsed = (
            time.perf_counter()
            - request_started
        )

        print(
            "[server] OpenAI response received"
        )

        print(
            f"[server] OpenAI time: "
            f"{openai_elapsed:.2f}s"
        )

        print(
            f"[server] Total time: "
            f"{total_elapsed:.2f}s"
        )

        print(
            f"[server] response_role="
            f"{message_data.get('role')}"
        )

        print(
            f"[server] tool_calls="
            f"{len(message_data.get('tool_calls', []))}"
        )

        print()

        return {
            "message":
                message_data
        }

    except RateLimitError as error:

        error_text = (
            f"OpenAI rate limit: {error}"
        )

        print(
            f"[server] {error_text}"
        )

        raise HTTPException(
            status_code=429,
            detail=error_text
        )

    except APITimeoutError as error:

        error_text = (
            f"OpenAI request timed out after "
            f"{OPENAI_TIMEOUT} seconds: {error}"
        )

        print(
            f"[server] {error_text}"
        )

        raise HTTPException(
            status_code=504,
            detail=error_text
        )

    except APIConnectionError as error:

        error_text = (
            f"Could not connect to OpenAI: {error}"
        )

        print(
            f"[server] {error_text}"
        )

        raise HTTPException(
            status_code=502,
            detail=error_text
        )

    except APIStatusError as error:

        error_text = (
            f"OpenAI API error "
            f"{error.status_code}: {error}"
        )

        print(
            f"[server] {error_text}"
        )

        raise HTTPException(
            status_code=500,
            detail=error_text
        )

    except HTTPException:

        raise

    except Exception as error:

        print(
            "[server] Unexpected error:"
        )

        traceback.print_exc()

        raise HTTPException(
            status_code=500,
            detail=str(error)
        )


# ============================================================
# Start
# ============================================================

if __name__ == "__main__":

    print()
    print(
        "=============================================="
    )

    print(
        "       My AI Coder API"
    )

    print(
        "=============================================="
    )

    print(
        f"Model: {MODEL}"
    )

    print(
        f"OpenAI timeout: {OPENAI_TIMEOUT}s"
    )

    print(
        f"OpenAI retries: {OPENAI_MAX_RETRIES}"
    )

    print(
        f"Server: http://{HOST}:{PORT}"
    )

    print(
        "=============================================="
    )

    print()

    uvicorn.run(
        "server:app",
        host=HOST,
        port=PORT,
        reload=True
    )