FROM ubuntu:22.10

RUN apt-get update && apt-get install --yes python3.11
RUN apt-get update && apt-get install --yes python3-pip

RUN pip install -U SPARQLWrapper
RUN pip install -U requests
RUN pip install -U flask

WORKDIR /wikidata_api

EXPOSE 5000

ENTRYPOINT python3 wikidata_api.py