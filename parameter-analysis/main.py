from robyn import Robyn

from parameter_analysis import ParameterAnalysisActor
from dapr_actor_interface.actor import DaprActor


app = Robyn(__file__)


actor = DaprActor(app)


async def startup_event():
    # Register ParameterAnalysisActor
    await actor.register_actor(ParameterAnalysisActor)


app.startup_handler(startup_event)
app.start(port=8080)
