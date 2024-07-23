FROM python:3.12.4

RUN pip install pipenv

COPY Pipfile* /
RUN pipenv install --ignore-pipfile && mkdir /app
COPY . /app

RUN ln -s $(pipenv --venv)/bin/gunicorn

ENTRYPOINT [ "./gunicorn" ]
CMD [ "-w", "4", "-b", "0.0.0.0:80", "app:api_app" ]
