FROM python:3.12

ADD src/requirements.txt .
RUN pip install -r requirements.txt

ADD src .

EXPOSE 8001

CMD ["gunicorn", "-c", "gunicorn.conf.py", "checker:app()"]
