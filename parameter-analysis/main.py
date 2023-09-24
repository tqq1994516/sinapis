from fastapi import FastAPI
from dapr.ext.fastapi import DaprActor

from parameter_analysis import ParameterAnalysisActor


app = FastAPI(title=f'{ParameterAnalysisActor.__name__}Service')


actor = DaprActor(app)


@app.on_event("startup")
async def startup_event():
    # Register ParameterAnalysisActor
    await actor.register_actor(ParameterAnalysisActor)
