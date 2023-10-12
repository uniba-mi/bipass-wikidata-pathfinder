FROM ubuntu:23.04

RUN apt-get update && apt-get install --yes python3.11
RUN apt-get update && apt-get install --yes python3-pip

# workaround for installing python packages outside of venv
RUN rm /usr/lib/python3.11/EXTERNALLY-MANAGED

RUN pip install -U SPARQLWrapper
RUN pip install -U requests
RUN pip install -U flask
RUN pip install -U beautifulsoup4

WORKDIR /wikidata_api

EXPOSE 5000

ENTRYPOINT python3 wikidata_api.py