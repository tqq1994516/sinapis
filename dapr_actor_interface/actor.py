import logging

from typing import Any, Type
from robyn import Robyn, jsonify, Response, Request
from robyn.robyn import HttpMethod
from robyn.router import Router
from dapr.serializers import DefaultJSONSerializer
from dapr.actor import Actor, ActorRuntime
from dapr.clients.exceptions import DaprInternalError, ERROR_CODE_UNKNOWN

DAPR_REENTRANCY_ID_HEADER = 'Dapr-Reentrancy-Id'
DEFAULT_HEADERS_CONTENT_TYPE = {"Content-Type": "application/json"}

logger = logging.getLogger(__name__)


def _wrap_response(
        status_code: int,
        msg: Any,
        error_code: str | None = None,
        content_type: dict | None = DEFAULT_HEADERS_CONTENT_TYPE):
    resp = None
    if isinstance(msg, str):
        response_obj = {
            'message': msg,
        }
        if not (status_code >= 200 and status_code < 300) and error_code:
            response_obj['errorCode'] = error_code
        resp = Response(
            description=jsonify(response_obj),
            headers=content_type,
            status_code=status_code
        )
    elif isinstance(msg, bytes):
        resp = Response(
            description=msg,
            headers=content_type,
            status_code=status_code
        )
    else:
        resp = jsonify(msg)
    return resp


class DaprActor(object):

    def __init__(
            self,
            app: Robyn,
            router_tags: list[str] | None = ['Actor']
    ):
        # router_tags should be added to all magic
        # Dapr Actor methods implemented here
        self._router_tags = router_tags
        self._router = Router()
        self._dapr_serializer = DefaultJSONSerializer()
        self.init_routes(self._router)
        app.router.routes.extend(self._router.routes)

    def init_routes(self, router: Router):

        async def healthz():
            return jsonify({'status': 'ok'})

        async def dapr_config():
            serialized = self._dapr_serializer.serialize(
                ActorRuntime.get_actor_config()
            )
            return _wrap_response(200, serialized)

        async def actor_deactivation(request: Request):
            try:
                actor_type_name = request.path_params["actor_type_name"]
                actor_id = request.path_params["actor_id"]
                await ActorRuntime.deactivate(actor_type_name, actor_id)
            except DaprInternalError as ex:
                return _wrap_response(
                    500,
                    ex.as_dict())
            except Exception as ex:
                return _wrap_response(
                    500,
                    repr(ex),
                    ERROR_CODE_UNKNOWN)

            msg = f'deactivated actor: {actor_type_name}.{actor_id}'
            logger.debug(msg)
            return _wrap_response(200, msg)

        async def actor_method(request: Request):
            try:
                # Read raw bytes from request stream
                actor_type_name = request.path_params["actor_type_name"]
                actor_id = request.path_params["actor_id"]
                method_name = request.path_params["method_name"]
                req_body = request.body
                reentrancy_id = request.headers.get(DAPR_REENTRANCY_ID_HEADER)
                result = await ActorRuntime.dispatch(
                    actor_type_name, actor_id, method_name, req_body, reentrancy_id) # type: ignore  # noqa
            except DaprInternalError as ex:
                return _wrap_response(
                    500, ex.as_dict())
            except Exception as ex:
                return _wrap_response(
                    500,
                    repr(ex),
                    ERROR_CODE_UNKNOWN)

            msg = f'called method. actor: {actor_type_name}.{actor_id}, method: {method_name}'  # noqa
            logger.debug(msg)
            return _wrap_response(200, result)

        async def actor_timer(request: Request):
            try:
                # Read raw bytes from request stream
                actor_type_name = request.path_params["actor_type_name"]
                actor_id = request.path_params["actor_id"]
                timer_name = request.path_params["timer_name"]
                req_body = request.body
                await ActorRuntime.fire_timer(actor_type_name, actor_id, timer_name, req_body)  # type: ignore # noqa
            except DaprInternalError as ex:
                return _wrap_response(
                    500,
                    ex.as_dict())
            except Exception as ex:
                return _wrap_response(
                    500,
                    repr(ex),
                    ERROR_CODE_UNKNOWN)

            msg = f'called timer. actor: {actor_type_name}.{actor_id}, timer: {timer_name}'  # noqa
            logger.debug(msg)
            return _wrap_response(200, msg)

        async def actor_reminder(request: Request):
            try:
                # Read raw bytes from request stream
                actor_type_name = request.path_params["actor_type_name"]
                actor_id = request.path_params["actor_id"]
                reminder_name = request.path_params["reminder_name"]
                req_body = request.body
                await ActorRuntime.fire_reminder(
                    actor_type_name, actor_id, reminder_name, req_body) # type: ignore
            except DaprInternalError as ex:
                return _wrap_response(
                    500,
                    ex.as_dict())
            except Exception as ex:
                return _wrap_response(
                    500,
                    repr(ex),
                    ERROR_CODE_UNKNOWN)

            msg = f'called reminder. actor: {actor_type_name}.{actor_id}, reminder: {reminder_name}'  # noqa
            logger.debug(msg)
            return _wrap_response(200, msg)

        router.add_route(HttpMethod.GET, "/healthz", healthz, True, None)
        router.add_route(
            HttpMethod.GET,
            "/dapr/config",
            dapr_config,
            True,
            None
        )
        router.add_route(
            HttpMethod.DELETE,
            "/actors/:actor_type_name/:actor_id",
            actor_deactivation,
            True,
            None
        )
        router.add_route(
            HttpMethod.PUT,
            "/actors/:actor_type_name/:actor_id/method/:method_name",
            actor_method,
            True,
            None
        )
        router.add_route(
            HttpMethod.PUT,
            "/actors/:actor_type_name/:actor_id/method/timer/:timer_name",
            actor_timer,
            True,
            None
        )
        router.add_route(
            HttpMethod.PUT,
            "/actors/:actor_type_name/:actor_id/method/remind/:reminder_name",
            actor_reminder,
            True,
            None
        )

    async def register_actor(self, actor: Type[Actor]) -> None:
        await ActorRuntime.register_actor(actor)
        logger.debug(
            f'registered actor: {actor.__class__.__name__}'  # type: ignore
        )
