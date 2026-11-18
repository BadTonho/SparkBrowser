# Spark 0.1.2

## New features

- Spark now tells you when something goes wrong: a card in the bottom-right corner shows the error, with **View error** to read the details and **Send error** to open the project's issues page already filled in. It covers internal errors, freezes, errors shown by the browser, script errors in the interface and in the new-tab page, and a previous run that did not close normally. Nothing is sent without your click, and page addresses and your user folder are removed from the report.

- Spark keeps a local error log (`logs/spark-errors.log` in the Spark data folder) with every error, in four levels (info, warning, error and critical), so you can attach it when reporting a bug. Only errors and critical errors show the card; failures Spark used to ignore silently are now logged as warnings.

## Fixes

- Fixed the audio background visual intensity slider getting stuck after lowering it, so it could no longer be raised.
