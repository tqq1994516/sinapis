from string import Template
from dapr.actor import Actor

from ..dapr_actor_interface import ParameterAnalysisActorInterface


class ParameterAnalysisActor(Actor, ParameterAnalysisActorInterface):
    """Implements Actor actor service
    """

    def __init__(self, ctx, actor_id):
        super(ParameterAnalysisActor, self).__init__(ctx, actor_id)

    async def _on_activate(self) -> None:
        """An callback which will be called whenever actor is activated."""
        print(f'Activate {self.__class__.__name__} actor!', flush=True)

    async def _on_deactivate(self) -> None:
        """An callback which will be called whenever actor is deactivated."""
        print(f'Deactivate {self.__class__.__name__} actor!', flush=True)

    async def template_parameter_analysis(self, data: str) -> list[str]:
        """An actor method which gets variable parameter in template.

        通过Template.get_identifiers方法获取字符串中`${}`包裹的变量参数
        """
        template = Template(data)
        return template.get_identifiers()
