FROM ubuntu:23.04

RUN apt-get update && apt-get install --yes python3.11
RUN apt-get update && apt-get install --yes python3-pip
RUN apt-get update && apt-get install --yes git
RUN apt-get update && apt-get install --yes curl

# workaround for installing python packages outside of venv
RUN rm /usr/lib/python3.11/EXTERNALLY-MANAGED

# install python packages
RUN pip install -U requests
RUN pip install -U bs4
RUN pip install -U torch
RUN pip install -U transformers
RUN pip install -U SPARQLWrapper
RUN pip install -U plotext
RUN pip install -U fire

WORKDIR /tmp

# download and extract genre
RUN curl -L https://github.com/facebookresearch/GENRE/archive/refs/tags/v0.1.3.tar.gz -o ./GENRE-0.1.3.tar.gz && tar -xvzf ./GENRE-0.1.3.tar.gz
RUN cp -r ./GENRE-0.1.3/genre /
RUN rm -rf ./GENRE-0.1.3.tar.gz && rm -rf ./GENRE-0.1.3

# download and extract huggingface model for genre
RUN curl -L http://dl.fbaipublicfiles.com/GENRE/hf_e2e_entity_linking_wiki_abs.tar.gz -o ./hf_e2e_entity_linking_wiki_abs.tar.gz && tar -xzf hf_e2e_entity_linking_wiki_abs.tar.gz
RUN mkdir -p /models/hf_e2e_entity_linking_wiki_abs
RUN cp -r ./hf_e2e_entity_linking_wiki_abs /models
RUN rm -rf ./hf_e2e_entity_linking_wiki_abs.tar.gz && rm -rf ./hf_e2e_entity_linking_wiki_abs

# download and extract TREC data
RUN mkdir /trec
RUN curl -L https://trec.nist.gov/data/million.query/07/07-million-query-topics.1-10000.gz -o ./07-million-query-topics.1-10000.gz && gzip -d ./07-million-query-topics.1-10000.gz
RUN mv ./07-million-query-topics.1-10000 /trec

RUN curl -L https://trec.nist.gov/data/million.query/08/08.million-query-topics.10001-20000.gz -o ./08.million-query-topics.10001-20000.gz && gzip -d ./08.million-query-topics.10001-20000.gz
RUN mv ./08.million-query-topics.10001-20000 /trec

RUN curl -L https://trec.nist.gov/data/million.query/09/09.mq.topics.20001-60000.gz -o ./09.mq.topics.20001-60000.gz && gzip -d ./09.mq.topics.20001-60000.gz
RUN mv ./09.mq.topics.20001-60000 /trec

# setting alias for running the factory
RUN echo 'alias factory="python3 -m src.query_factory launch_query_factory"' >> ~/.bashrc

WORKDIR /

ENTRYPOINT bash