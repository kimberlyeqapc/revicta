"""Query Envelopes."""

from typing import Optional, Union

from vitruvia.communication import MsgStatus, send_sync_message
from vitruvia.crypto import generate_user_address
from vitruvia.envelope import Envelope
from vitruvia.models import Model
from vitruvia.resolver import Resolver


async def query(
    destination: str,
    message: Model,
    resolver: Optional[Resolver] = None,
    timeout: int = 30,
) -> Union[MsgStatus, Envelope]:
    """
    Query a remote agent with a message and retrieve the response envelope.

    Args:
        destination (str): The destination address of the remote agent.
        message (Model): The message to send.
        resolver (Optional[Resolver], optional): The resolver to use for endpoint resolution.
        Defaults to GlobalResolver.
        timeout (int): The timeout for the query in seconds. Defaults to 30.

    Returns:
        Union[MsgStatus, Envelope]: The response envelope if successful, otherwise MsgStatus.
    """
    response = await send_sync_message(
        destination=destination,
        message=message,
        response_type=None,
        sender=generate_user_address(),
        resolver=resolver,
        timeout=timeout,
    )
    if isinstance(response, (MsgStatus, Envelope)):
        return response
    raise ValueError("Invalid response received.")