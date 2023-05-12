FROM ubuntu:23.04

RUN apt-get update && apt-get install --yes python3.11
RUN apt-get update && apt-get install --yes python3-pip

# workaround for installing python packages outside of venv
RUN rm /usr/lib/python3.11/EXTERNALLY-MANAGED

RUN pip install -U numpy
RUN pip install -U scipy
RUN pip install -U flask
RUN pip install -U sentence-transformers
RUN pip install -U fire

WORKDIR /wembed_api

EXPOSE 5000

ENTRYPOINT python3 wembed_api.py
