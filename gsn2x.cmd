@echo off

set input=%2
REM default to PNG
if [%input%] == [] (set input=png) 

python yslt.py -s gsn2dot.yslt "%1" | dot -T%input% > "%~dpn1.%input%"