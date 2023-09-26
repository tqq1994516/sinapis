from abc import abstractmethod
from dapr.actor import ActorInterface, actormethod


class ParameterAnalysisActorInterface(ActorInterface):
    @abstractmethod
    @actormethod(name="TemplateParameterAnalysis")
    async def template_paramter_analysis(self, data: str) -> list[str]:
        ...
