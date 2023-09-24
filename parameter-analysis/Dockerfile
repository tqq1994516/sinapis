FROM python:3.11.2-bullseye
USER root

COPY pip.conf /root/.pip/pip.conf

RUN mkdir -p /parameter_analysis_actor

RUN python -m pip install --upgrade pip

WORKDIR /parameter_analysis_actor
ENV PYTHONPATH /parameter_analysis_actor
COPY . .

RUN pip install -r requirements.txt
EXPOSE 8000
ENTRYPOINT ["python"]