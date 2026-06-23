10 REM LID-TRAINER TEIL 2 VON 3
12 REM COPYRIGHT (C) 2026 R. COIGNARD
14 CLEAR 4000:WINDOW
16 DIM O$(4):DIM V(4):DIM OZ(4):DIM P(99):DG$="1234"
18 GOTO 100
100 BORDER 5:PAPER 5:INK 8:CLS
102 PRINT AT(2,10);"LEBEN IN DEUTSCHLAND"
104 PRINT AT(4,12);"PRUEFUNGSTRAINER"
106 PRINT AT(6,8);"TEIL 2 VON 3   99 FRAGEN"
108 PRINT AT(11,6);"1. LERNEN  - ALLE FRAGEN"
110 PRINT AT(13,6);"2. TEST    - MIT ZUFALLSZAHL"
112 PRINT AT(23,5);"COPYRIGHT (C) 2026 R. COIGNARD"
116 K$=INKEY$:IF K$="1" THEN 200
118 IF K$="2" THEN 300
120 GOTO 116
200 S=0
202 FOR I=1 TO 4:V(I)=I:NEXT
204 FOR QQ=1 TO 99
206 KK=QQ:GOSUB 1000
208 CP=0:FOR M=1 TO 4:IF V(M)=C THEN CP=M
210 NEXT M
212 HE$="TEIL 2  FRAGE "+MID$(STR$(QQ),2)+"/99  RICHTIG ":HD$=HE$+MID$(STR$(S),2)
214 GOSUB 1100
216 GOSUB 1400
218 GOSUB 1600
220 IF AC=2 THEN 100
222 NEXT QQ
224 GOTO 100
300 S=0
302 BORDER 5:PAPER 5:INK 8:CLS
304 PRINT AT(2,4);"TESTMODUS - TEIL 2"
306 PRINT AT(5,4);"ZUFALLSZAHL 0=ZUFALL:"
308 INPUT SE
310 IF SE<0 THEN SE=-SE
312 IF SE=0 THEN SE=INT(RND(1)*9000)+1000
314 H9$="GEWAEHLT: "+MID$(STR$(SE),2):PRINT AT(8,4);H9$
316 PRINT AT(9,4);"WIEVIEL FRAGEN 1-99:"
318 INPUT TC
320 IF TC<1 THEN TC=1
322 IF TC>99 THEN TC=99
324 RR=SE-INT(SE/65536)*65536
326 FOR I=1 TO 99:P(I)=I:NEXT
328 FOR I=99 TO 2 STEP -1:GOSUB 1500:J=INT(RV*I)+1:T=P(I):P(I)=P(J):P(J)=T:NEXT
330 FOR QI=1 TO TC
332 KK=P(QI):GOSUB 1000
334 FOR I=1 TO 4:V(I)=I:NEXT
336 FOR I=4 TO 2 STEP -1:GOSUB 1500:J=INT(RV*I)+1:T=V(I):V(I)=V(J):V(J)=T:NEXT
338 CP=0:FOR M=1 TO 4:IF V(M)=C THEN CP=M
340 NEXT M
342 HE$="TEIL 2  FRAGE "+MID$(STR$(QI),2)+"/"+MID$(STR$(TC),2)+"  RICHTIG ":HD$=HE$+MID$(STR$(S),2)
344 GOSUB 1100
346 GOSUB 1400
348 GOSUB 1600
350 IF AC=2 THEN 100
352 NEXT QI
354 BORDER 5:PAPER 5:INK 8:CLS
356 PRINT AT(4,6);"ERGEBNIS - TEIL 2"
358 H9$="RICHTIG: "+MID$(STR$(S),2)+" VON "+MID$(STR$(TC),2):PRINT AT(7,6);H9$
360 PZ=INT(S/TC*100+.5):H9$="QUOTE: "+MID$(STR$(PZ),2)+" PROZENT":PRINT AT(9,6);H9$
362 IF S*2>=TC THEN PRINT AT(11,6);"BESTANDEN"
364 IF S*2<TC THEN PRINT AT(11,6);"NICHT BESTANDEN"
366 H9$="ZUFALLSZAHL: "+MID$(STR$(SE),2):PRINT AT(13,6);H9$
368 PRINT AT(20,6);"LEER / ENTER = MENUE"
370 GOSUB 1450:GOTO 100
1000 RESTORE
1002 IF KK<=1 THEN 1008
1004 FOR I=1 TO 6*(KK-1):READ D$:NEXT
1008 READ Q$,C
1010 FOR I=1 TO 4:READ O$(I):NEXT
1012 RETURN
1100 BORDER 5:PAPER 5:INK 8:CLS
1102 PRINT AT(0,0);HD$
1104 PRINT AT(1,0);STRING$(39,"-")
1106 RW=3:CL=0:W=37:T$=Q$:GOSUB 1200
1108 RW=RW+1
1110 FOR M=1 TO 4
1112 OZ(M)=RW
1114 PAPER 5:INK 8:P9$=MID$(DG$,M,1)+".":PRINT AT(RW,0);P9$
1116 CL=3:W=34:T$=O$(V(M)):GOSUB 1200
1118 NEXT M
1120 RETURN
1200 PP=1:LL=LEN(T$)
1202 IF PP>LL THEN RETURN
1204 IF LL-PP+1<=W THEN PRINT AT(RW,CL);MID$(T$,PP):RW=RW+1:RETURN
1206 BK=0
1208 FOR I=PP+W TO PP+1 STEP -1:IF MID$(T$,I,1)=" " THEN BK=I:I=PP+1
1210 NEXT I
1212 IF BK=0 THEN SG$=MID$(T$,PP,W):NP=PP+W:GOTO 1216
1214 SG$=MID$(T$,PP,BK-PP):NP=BK+1
1216 PRINT AT(RW,CL);SG$:RW=RW+1:PP=NP:GOTO 1202
1320 CL=3:W=34:T$=O$(V(M)):GOSUB 1360:RW=OZ(M):GOSUB 1380:RETURN
1360 ML=0:PP=1:LL=LEN(T$)
1361 IF PP>LL THEN RETURN
1362 IF LL-PP+1>W THEN 1366
1363 GL=LL-PP+1:IF GL>ML THEN ML=GL
1364 RETURN
1366 BK=0:FOR I=PP+W TO PP+1 STEP -1:IF MID$(T$,I,1)=" " THEN BK=I:I=PP+1
1367 NEXT I
1368 IF BK=0 THEN GL=W:NP=PP+W
1369 IF BK>0 THEN GL=BK-PP:NP=BK+1
1370 IF GL>ML THEN ML=GL
1371 PP=NP:GOTO 1361
1380 PP=1:LL=LEN(T$)
1381 IF PP>LL THEN RETURN
1382 IF LL-PP+1<=W THEN SG$=MID$(T$,PP):GOSUB 1395:RETURN
1383 BK=0:FOR I=PP+W TO PP+1 STEP -1:IF MID$(T$,I,1)=" " THEN BK=I:I=PP+1
1384 NEXT I
1385 IF BK=0 THEN SG$=MID$(T$,PP,W):NP=PP+W
1386 IF BK>0 THEN SG$=MID$(T$,PP,BK-PP):NP=BK+1
1387 GOSUB 1395:PP=NP:GOTO 1381
1395 PT$=SG$:IF ML>LEN(SG$) THEN PT$=SG$+STRING$(ML-LEN(SG$)," ")
1396 PRINT AT(RW,CL);PT$:RW=RW+1:RETURN
1400 K$=INKEY$
1402 IF K$="1" THEN AK=1:RETURN
1403 IF K$="2" THEN AK=2:RETURN
1404 IF K$="3" THEN AK=3:RETURN
1405 IF K$="4" THEN AK=4:RETURN
1406 GOTO 1400
1450 K$=INKEY$
1452 IF K$=" " THEN AC=1:RETURN
1454 IF K$=CHR$(13) THEN AC=1:RETURN
1456 IF K$="0" THEN AC=2:RETURN
1458 GOTO 1450
1500 RR=RR*2053+13849:RR=RR-INT(RR/65536)*65536:RV=RR/65536:RETURN
1600 IF AK=CP THEN 1650
1602 BEEP
1604 PAPER 2:INK 5:M=AK:GOSUB 1320:PAPER 5:INK 8
1606 PAPER 3:INK 5:M=CP:GOSUB 1320:PAPER 5:INK 8
1608 PRINT AT(22,0);"FALSCH  LEER/ENTER=WEITER 0=MENUE"
1610 GOSUB 1450:RETURN
1650 S=S+1:BEEP:PAPER 5:INK 8:PRINT AT(0,0);HE$+MID$(STR$(S),2)
1652 PAPER 3:INK 5:M=AK:GOSUB 1320:PAPER 5:INK 8
1654 PRINT AT(22,0);"RICHTIG LEER/ENTER=WEITER 0=MENUE"
1656 GOSUB 1450:RETURN
2000 DATA "Gewerkschaften sind Interessenverbaende der ...",2
2001 DATA "Jugendlichen."
2002 DATA "Arbeitnehmer und Arbeitnehmerinnen."
2003 DATA "Rentner und Rentnerinnen."
2004 DATA "Arbeitgeber und Arbeitgeberinnen."
2005 DATA "Womit kann man in der Bundesrepublik Deutschland geehrt werden, wenn man auf politischem, wirtschaftlichem, kulturellem, geistigem oder sozialem Gebiet eine besondere Leistung erbracht hat? Mit dem ...",2
2006 DATA "Bundesadler"
2007 DATA "Bundesverdienstkreuz"
2008 DATA "Vaterlaendischen Verdienstorden"
2009 DATA "Ehrentitel 'Held der Deutschen Demokratischen Republik'"
2010 DATA "Was wird in Deutschland als 'Ampelkoalition' bezeichnet?",2
2011 DATA "der Bundestagsfraktionen von CDU und CSU"
2012 DATA "von SPD, FDP und Buendnis 90/Die Gruenen in einer Regierung"
2013 DATA "von CSU, Die Linke und Buendnis 90/Die Gruenen in einer Regierung"
2014 DATA "der Bundestagsfraktionen von CDU und SPD"
2015 DATA "Eine Frau in Deutschland verliert ihre Arbeit. Was darf nicht der Grund fuer diese Entlassung sein?",4
2016 DATA "Die Frau ist lange krank und arbeitsunfaehig."
2017 DATA "Die Frau kam oft zu spaet zur Arbeit."
2018 DATA "Die Frau erledigt private Sachen waehrend der Arbeitszeit."
2019 DATA "Die Frau bekommt ein Kind und ihr Chef weiss das."
2020 DATA "Was ist eine Aufgabe von Wahlhelfern / Wahlhelferinnen in Deutschland?",4
2021 DATA "Sie helfen alten Menschen bei der Stimmabgabe in der Wahlkabine."
2022 DATA "Sie schreiben die Wahlbenachrichtigungen vor der Wahl."
2023 DATA "Sie geben Zwischenergebnisse an die Medien weiter."
2024 DATA "Sie zaehlen die Stimmen nach dem Ende der Wahl."
2025 DATA "In Deutschland helfen ehrenamtliche Wahlhelfer und Wahlhelferinnen bei den Wahlen. Was ist eine Aufgabe von Wahlhelfern / Wahlhelferinnen?",4
2026 DATA "Sie helfen Kindern und alten Menschen beim Waehlen."
2027 DATA "Sie schreiben Karten und Briefe mit der Angabe des Wahllokals."
2028 DATA "Sie geben Zwischenergebnisse an Journalisten weiter."
2029 DATA "Sie zaehlen die Stimmen nach dem Ende der Wahl."
2030 DATA "Fuer wie viele Jahre wird der Bundestag in Deutschland gewaehlt?",2
2031 DATA "2 Jahre"
2032 DATA "4 Jahre"
2033 DATA "6 Jahre"
2034 DATA "8 Jahre"
2035 DATA "Bei einer Bundestagswahl in Deutschland darf jeder waehlen, der ...",2
2036 DATA "in der Bundesrepublik Deutschland wohnt und waehlen moechte."
2037 DATA "Buerger / Buergerin der Bundesrepublik Deutschland ist und mindestens 18 Jahre alt ist."
2038 DATA "seit mindestens 3 Jahren in der Bundesrepublik Deutschland lebt."
2039 DATA "Buerger / Buergerin der Bundesrepublik Deutschland ist und mindestens 21 Jahre alt ist."
2040 DATA "Wie oft gibt es normalerweise Bundestagswahlen in Deutschland?",2
2041 DATA "alle drei Jahre"
2042 DATA "alle vier Jahre"
2043 DATA "alle fuenf Jahre"
2044 DATA "alle sechs Jahre"
2045 DATA "Fuer wie viele Jahre wird der Bundestag in Deutschland gewaehlt?",3
2046 DATA "2 Jahre"
2047 DATA "3 Jahre"
2048 DATA "4 Jahre"
2049 DATA "5 Jahre"
2050 DATA "Welche Handlung mit Bezug auf den Staat Israel ist in Deutschland verboten?",4
2051 DATA "Die Politik Israels oeffentlich kritisieren."
2052 DATA "Das Aufhaengen einer israelischen Flagge auf dem Privatgrundstueck."
2053 DATA "Eine Diskussion ueber die Politik Israels."
2054 DATA "Der oeffentliche Aufruf zur Vernichtung Israels."
2055 DATA "Die Wahlen in Deutschland sind ...",2
2056 DATA "speziell."
2057 DATA "geheim."
2058 DATA "berufsbezogen."
2059 DATA "geschlechtsabhaengig."
2060 DATA "Wahlen in Deutschland gewinnt die Partei, die ...",1
2061 DATA "die meisten Stimmen bekommt."
2062 DATA "die meisten Maenner mehrheitlich gewaehlt haben."
2063 DATA "die meisten Stimmen bei den Arbeitern / Arbeiterinnen bekommen hat."
2064 DATA "die meisten Erststimmen fuer ihren Kanzlerkandidaten / ihre Kanzlerkandidatin erhalten hat."
2065 DATA "An demokratischen Wahlen in Deutschland teilzunehmen ist ...",2
2066 DATA "eine Pflicht."
2067 DATA "ein Recht."
2068 DATA "ein Zwang."
2069 DATA "eine Last."
2070 DATA "Was bedeutet 'aktives Wahlrecht' in Deutschland?",3
2071 DATA "Man kann gewaehlt werden."
2072 DATA "Man muss waehlen gehen."
2073 DATA "Man kann waehlen."
2074 DATA "Man muss zur Auszaehlung der Stimmen gehen."
2075 DATA "Wenn Sie bei einer Bundestagswahl in Deutschland waehlen duerfen, heisst das ...",4
2076 DATA "aktive Wahlkampagne."
2077 DATA "aktives Wahlverfahren."
2078 DATA "aktiver Wahlkampf."
2079 DATA "aktives Wahlrecht."
2080 DATA "Wie viel Prozent der Zweitstimmen muessen Parteien mindestens bekommen, um in den Deutschen Bundestag gewaehlt zu werden?",3
2081 DATA "3 %"
2082 DATA "4 %"
2083 DATA "5 %"
2084 DATA "6 %"
2085 DATA "Wer darf bei den rund 40 juedischen Makkabi-Sportvereinen Mitglied werden?",4
2086 DATA "nur Deutsche"
2087 DATA "nur Israelis"
2088 DATA "nur religioese Menschen"
2089 DATA "alle Menschen"
2090 DATA "Wahlen in Deutschland sind frei. Was bedeutet das?",3
2091 DATA "Alle verurteilten Straftaeter / Straftaeterinnen duerfen nicht waehlen."
2092 DATA "Wenn ich waehlen gehen moechte, muss mein Arbeitgeber / meine Arbeitgeberin mir frei geben."
2093 DATA "Jede Person kann ohne Zwang entscheiden, ob sie waehlen moechte und wen sie waehlen moechte."
2094 DATA "Ich kann frei entscheiden, wo ich waehlen gehen moechte."
2095 DATA "Das Wahlsystem in Deutschland ist ein ...",3
2096 DATA "Zensuswahlrecht."
2097 DATA "Dreiklassenwahlrecht."
2098 DATA "Mehrheits- und Verhaeltniswahlrecht."
2099 DATA "allgemeines Maennerwahlrecht."
2100 DATA "Eine Partei moechte in den Deutschen Bundestag. Sie muss aber einen Mindestanteil an Waehlerstimmen haben. Das heisst ...",1
2101 DATA "5 %-Huerde."
2102 DATA "Zulassungsgrenze."
2103 DATA "Basiswert."
2104 DATA "Richtlinie."
2105 DATA "Welchem Grundsatz unterliegen Wahlen in Deutschland? Wahlen in Deutschland sind ...",1
2106 DATA "frei, gleich, geheim."
2107 DATA "offen, sicher, frei."
2108 DATA "geschlossen, gleich, sicher."
2109 DATA "sicher, offen, freiwillig."
2110 DATA "Was ist in Deutschland die '5 %-Huerde'?",3
2111 DATA "Abstimmungsregelung im Bundestag fuer kleine Parteien"
2112 DATA "Anwesenheitskontrolle im Bundestag fuer Abstimmungen"
2113 DATA "Mindestanteil an Waehlerstimmen, um ins Parlament zu kommen"
2114 DATA "Anwesenheitskontrolle im Bundesrat fuer Abstimmungen"
2115 DATA "Die Bundestagswahl in Deutschland ist die Wahl ...",3
2116 DATA "des Bundeskanzlers / der Bundeskanzlerin."
2117 DATA "der Parlamente der Laender."
2118 DATA "des Parlaments fuer Deutschland."
2119 DATA "des Bundespraesidenten / der Bundespraesidentin."
2120 DATA "In einer Demokratie ist eine Funktion von regelmaessigen Wahlen, ...",2
2121 DATA "die Buerger und Buergerinnen zu zwingen, ihre Stimme abzugeben."
2122 DATA "nach dem Willen der Waehlermehrheit den Wechsel der Regierung zu ermoeglichen."
2123 DATA "im Land bestehende Gesetze beizubehalten."
2124 DATA "den Armen mehr Macht zu geben."
2125 DATA "Was bekommen wahlberechtigte Buerger und Buergerinnen in Deutschland vor einer Wahl?",1
2126 DATA "eine Wahlbenachrichtigung von der Gemeinde"
2127 DATA "eine Wahlerlaubnis vom Bundespraesidenten / von der Bundespraesidentin"
2128 DATA "eine Benachrichtigung von der Bundesversammlung"
2129 DATA "eine Benachrichtigung vom Pfarramt"
2130 DATA "Warum gibt es die 5 %-Huerde im Wahlgesetz der Bundesrepublik Deutschland? Es gibt sie, weil ...",3
2131 DATA "die Programme von vielen kleinen Parteien viele Gemeinsamkeiten haben."
2132 DATA "die Buerger und Buergerinnen bei vielen kleinen Parteien die Orientierung verlieren koennen."
2133 DATA "viele kleine Parteien die Regierungsbildung erschweren."
2134 DATA "die kleinen Parteien nicht so viel Geld haben, um die Politiker und Politikerinnen zu bezahlen."
2135 DATA "Parlamentsmitglieder, die von den Buergern und Buergerinnen gewaehlt werden, nennt man ...",1
2136 DATA "Abgeordnete."
2137 DATA "Kanzler / Kanzlerinnen."
2138 DATA "Botschafter / Botschafterinnen."
2139 DATA "Ministerpraesidenten / Ministerpraesidentinnen."
2140 DATA "Vom Volk gewaehlt wird in Deutschland ...",3
2141 DATA "der Bundeskanzler / die Bundeskanzlerin."
2142 DATA "der Ministerpraesident / die Ministerpraesidentin eines Bundeslandes."
2143 DATA "der Bundestag."
2144 DATA "der Bundespraesident / die Bundespraesidentin."
2145 DATA "In Deutschland ist ein Buergermeister / eine Buergermeisterin ...",3
2146 DATA "der Leiter / die Leiterin einer Schule."
2147 DATA "der Chef / die Chefin einer Bank."
2148 DATA "das Oberhaupt einer Gemeinde."
2149 DATA "der / die Vorsitzende einer Partei."
2150 DATA "Viele Menschen in Deutschland arbeiten in ihrer Freizeit ehrenamtlich. Was bedeutet das?",2
2151 DATA "Sie arbeiten als Soldaten / Soldatinnen."
2152 DATA "Sie arbeiten freiwillig und unbezahlt in Vereinen und Verbaenden."
2153 DATA "Sie arbeiten in der Bundesregierung."
2154 DATA "Sie arbeiten in einem Krankenhaus und verdienen dabei Geld."
2155 DATA "Was ist bei Bundestags- und Landtagswahlen in Deutschland erlaubt?",2
2156 DATA "Der Ehemann waehlt fuer seine Frau mit."
2157 DATA "Man kann durch Briefwahl seine Stimme abgeben."
2158 DATA "Man kann am Wahltag telefonisch seine Stimme abgeben."
2159 DATA "Kinder ab dem Alter von 14 Jahren duerfen waehlen."
2160 DATA "Man will die Buslinie abschaffen, mit der Sie immer zur Arbeit fahren. Was koennen Sie machen, um die Buslinie zu erhalten?",1
2161 DATA "Ich beteilige mich an einer Buergerinitiative fuer die Erhaltung der Buslinie oder gruende selber eine Initiative."
2162 DATA "Ich werde Mitglied in einem Sportverein und trainiere Rad fahren."
2163 DATA "Ich wende mich an das Finanzamt, weil ich als Steuerzahler / Steuerzahlerin ein Recht auf die Buslinie habe."
2164 DATA "Ich schreibe einen Brief an das Forstamt der Gemeinde."
2165 DATA "Wen vertreten die Gewerkschaften in Deutschland?",4
2166 DATA "grosse Unternehmen"
2167 DATA "kleine Unternehmen"
2168 DATA "Selbststaendige"
2169 DATA "Arbeitnehmer und Arbeitnehmerinnen"
2170 DATA "Sie gehen in Deutschland zum Arbeitsgericht bei ...",2
2171 DATA "falscher Nebenkostenabrechnung."
2172 DATA "ungerechtfertigter Kuendigung durch Ihren Chef / Ihre Chefin."
2173 DATA "Problemen mit den Nachbarn / Nachbarinnen."
2174 DATA "Schwierigkeiten nach einem Verkehrsunfall."
2175 DATA "Welches Gericht ist in Deutschland bei Konflikten in der Arbeitswelt zustaendig?",3
2176 DATA "das Familiengericht"
2177 DATA "das Strafgericht"
2178 DATA "das Arbeitsgericht"
2179 DATA "das Amtsgericht"
2180 DATA "Was kann ich in Deutschland machen, wenn mir mein Arbeitgeber / meine Arbeitgeberin zu Unrecht gekuendigt hat?",3
2181 DATA "weiter arbeiten und freundlich zum Chef / zur Chefin sein"
2182 DATA "ein Mahnverfahren gegen den Arbeitgeber / die Arbeitgeberin fuehren"
2183 DATA "Kuendigungsschutzklage erheben"
2184 DATA "den Arbeitgeber / die Arbeitgeberin bei der Polizei anzeigen"
2185 DATA "Wann kommt es in Deutschland zu einem Prozess vor Gericht? Wenn jemand ...",2
2186 DATA "zu einer anderen Religion uebertritt."
2187 DATA "eine Straftat begangen hat und angeklagt wird."
2188 DATA "eine andere Meinung als die der Regierung vertritt."
2189 DATA "sein Auto falsch geparkt hat und es abgeschleppt wird."
2190 DATA "Was macht ein Schoeffe / eine Schoeffin in Deutschland? Er / Sie ...",1
2191 DATA "entscheidet mit Richtern / Richterinnen ueber Schuld und Strafe."
2192 DATA "gibt Buergern / Buergerinnen rechtlichen Rat."
2193 DATA "stellt Urkunden aus."
2194 DATA "verteidigt den Angeklagten / die Angeklagte."
2195 DATA "Wer beraet in Deutschland Personen bei Rechtsfragen und vertritt sie vor Gericht?",1
2196 DATA "ein Rechtsanwalt / eine Rechtsanwaeltin"
2197 DATA "ein Richter / eine Richterin"
2198 DATA "ein Schoeffe / eine Schoeffin"
2199 DATA "ein Staatsanwalt / eine Staatsanwaeltin"
2200 DATA "Was ist die Hauptaufgabe eines Richters / einer Richterin in Deutschland? Ein Richter / eine Richterin ...",2
2201 DATA "vertritt Buerger und Buergerinnen vor einem Gericht."
2202 DATA "arbeitet an einem Gericht und spricht Urteile."
2203 DATA "aendert Gesetze."
2204 DATA "betreut Jugendliche vor Gericht."
2205 DATA "Ein Richter / eine Richterin in Deutschland gehoert zur ...",1
2206 DATA "Judikative."
2207 DATA "Exekutive."
2208 DATA "Operative."
2209 DATA "Legislative."
2210 DATA "Ein Richter / eine Richterin gehoert in Deutschland zur ...",2
2211 DATA "vollziehenden Gewalt."
2212 DATA "rechtsprechenden Gewalt."
2213 DATA "planenden Gewalt."
2214 DATA "gesetzgebenden Gewalt."
2215 DATA "In Deutschland wird die Staatsgewalt geteilt. Fuer welche Staatsgewalt arbeitet ein Richter / eine Richterin? Fuer die ...",1
2216 DATA "Judikative."
2217 DATA "Exekutive."
2218 DATA "Presse."
2219 DATA "Legislative."
2220 DATA "Wie nennt man in Deutschland ein Verfahren vor einem Gericht?",4
2221 DATA "Programm"
2222 DATA "Prozedur"
2223 DATA "Protokoll"
2224 DATA "Prozess"
2225 DATA "Was ist die Arbeit eines Richters / einer Richterin in Deutschland?",2
2226 DATA "Deutschland regieren"
2227 DATA "Recht sprechen"
2228 DATA "Plaene erstellen"
2229 DATA "Gesetze erlassen"
2230 DATA "Was ist eine Aufgabe der Polizei in Deutschland?",4
2231 DATA "das Land zu verteidigen"
2232 DATA "die Buergerinnen und Buerger abzuhoeren"
2233 DATA "die Gesetze zu beschliessen"
2234 DATA "die Einhaltung von Gesetzen zu ueberwachen"
2235 DATA "Was ist ein Beispiel fuer antisemitisches Verhalten?",3
2236 DATA "ein juedisches Fest besuchen"
2237 DATA "die israelische Regierung kritisieren"
2238 DATA "den Holocaust leugnen"
2239 DATA "gegen Juden Fussball spielen"
2240 DATA "Ein Gerichtsschoeffe / eine Gerichtsschoeffin in Deutschland ist ...",2
2241 DATA "der Stellvertreter / die Stellvertreterin des Stadtoberhaupts."
2242 DATA "ein ehrenamtlicher Richter / eine ehrenamtliche Richterin."
2243 DATA "ein Mitglied eines Gemeinderats."
2244 DATA "eine Person, die Jura studiert hat."
2245 DATA "Wer baute die Mauer in Berlin?",3
2246 DATA "Grossbritannien"
2247 DATA "die Bundesrepublik Deutschland"
2248 DATA "die DDR"
2249 DATA "die USA"
2250 DATA "Wann waren die Nationalsozialisten mit Adolf Hitler in Deutschland an der Macht?",3
2251 DATA "1918 bis 1923"
2252 DATA "1932 bis 1950"
2253 DATA "1933 bis 1945"
2254 DATA "1945 bis 1989"
2255 DATA "Was war am 8. Mai 1945?",4
2256 DATA "Tod Adolf Hitlers"
2257 DATA "Beginn des Berliner Mauerbaus"
2258 DATA "Wahl von Konrad Adenauer zum Bundeskanzler"
2259 DATA "Ende des Zweiten Weltkriegs in Europa"
2260 DATA "Wann war der Zweite Weltkrieg zu Ende?",2
2261 DATA "1933"
2262 DATA "1945"
2263 DATA "1949"
2264 DATA "1961"
2265 DATA "Wann waren die Nationalsozialisten in Deutschland an der Macht?",3
2266 DATA "1888 bis 1918"
2267 DATA "1921 bis 1934"
2268 DATA "1933 bis 1945"
2269 DATA "1949 bis 1963"
2270 DATA "In welchem Jahr wurde Hitler Reichskanzler?",3
2271 DATA "1923"
2272 DATA "1927"
2273 DATA "1933"
2274 DATA "1936"
2275 DATA "Die Nationalsozialisten mit Adolf Hitler errichteten 1933 in Deutschland ...",1
2276 DATA "eine Diktatur."
2277 DATA "einen demokratischen Staat."
2278 DATA "eine Monarchie."
2279 DATA "ein Fuerstentum."
2280 DATA "Das 'Dritte Reich' war eine ...",1
2281 DATA "Diktatur."
2282 DATA "Demokratie."
2283 DATA "Monarchie."
2284 DATA "Raeterepublik."
2285 DATA "Was gab es in Deutschland nicht waehrend der Zeit des Nationalsozialismus?",1
2286 DATA "freie Wahlen"
2287 DATA "Pressezensur"
2288 DATA "willkuerliche Verhaftungen"
2289 DATA "Verfolgung der Juden"
2290 DATA "Welcher Krieg dauerte von 1939 bis 1945?",2
2291 DATA "der Erste Weltkrieg"
2292 DATA "der Zweite Weltkrieg"
2293 DATA "der Vietnamkrieg"
2294 DATA "der Golfkrieg"
2295 DATA "Was kennzeichnete den NS-Staat? Eine Politik ...",1
2296 DATA "des staatlichen Rassismus"
2297 DATA "der Meinungsfreiheit"
2298 DATA "der allgemeinen Religionsfreiheit"
2299 DATA "der Entwicklung der Demokratie"
2300 DATA "Claus Schenk Graf von Stauffenberg wurde bekannt durch ...",4
2301 DATA "eine Goldmedaille bei den Olympischen Spielen 1936."
2302 DATA "den Bau des Reichstagsgebaeudes."
2303 DATA "den Aufbau der Wehrmacht."
2304 DATA "das Attentat auf Hitler am 20. Juli 1944."
2305 DATA "In welchem Jahr zerstoerten die Nationalsozialisten Synagogen und juedische Geschaefte in Deutschland?",3
2306 DATA "1925"
2307 DATA "1930"
2308 DATA "1938"
2309 DATA "1945"
2310 DATA "Was passierte am 9. November 1938 in Deutschland?",3
2311 DATA "Mit dem Angriff auf Polen beginnt der Zweite Weltkrieg."
2312 DATA "Die Nationalsozialisten verlieren eine Wahl und loesen den Reichstag auf."
2313 DATA "Juedische Geschaefte und Synagogen werden durch Nationalsozialisten und ihre Anhaenger zerstoert."
2314 DATA "Hitler wird Reichspraesident und laesst alle Parteien verbieten."
2315 DATA "Wie hiess der erste Bundeskanzler der Bundesrepublik Deutschland?",1
2316 DATA "Konrad Adenauer"
2317 DATA "Kurt Georg Kiesinger"
2318 DATA "Helmut Schmidt"
2319 DATA "Willy Brandt"
2320 DATA "Bei welchen Demonstrationen in Deutschland riefen die Menschen 'Wir sind das Volk'?",1
2321 DATA "bei den Montagsdemonstrationen 1989 in der DDR"
2322 DATA "beim Arbeiteraufstand 1953 in der DDR"
2323 DATA "bei den Demonstrationen 1968 in der Bundesrepublik Deutschland"
2324 DATA "bei den Anti-Atomkraft-Demonstrationen 1985 in der Bundesrepublik Deutschland"
2325 DATA "Welche Laender wurden nach dem Zweiten Weltkrieg in Deutschland als 'Alliierte Besatzungsmaechte' bezeichnet?",4
2326 DATA "Sowjetunion, Grossbritannien, Polen, Schweden"
2327 DATA "Frankreich, Sowjetunion, Italien, Japan"
2328 DATA "USA, Sowjetunion, Spanien, Portugal"
2329 DATA "USA, Sowjetunion, Grossbritannien, Frankreich"
2330 DATA "Welches Land war keine 'Alliierte Besatzungsmacht' in Deutschland?",4
2331 DATA "USA"
2332 DATA "Sowjetunion"
2333 DATA "Frankreich"
2334 DATA "Japan"
2335 DATA "Wann wurde die Bundesrepublik Deutschland gegruendet?",3
2336 DATA "1939"
2337 DATA "1945"
2338 DATA "1949"
2339 DATA "1951"
2340 DATA "Was gab es waehrend der Zeit des Nationalsozialismus in Deutschland?",3
2341 DATA "das Recht zur freien Entfaltung der Persoenlichkeit"
2342 DATA "Pressefreiheit"
2343 DATA "das Verbot von Parteien"
2344 DATA "den Schutz der Menschenwuerde"
2345 DATA "Soziale Marktwirtschaft bedeutet, die Wirtschaft ...",4
2346 DATA "steuert sich allein nach Angebot und Nachfrage."
2347 DATA "wird vom Staat geplant und gesteuert, Angebot und Nachfrage werden nicht beruecksichtigt."
2348 DATA "richtet sich nach der Nachfrage im Ausland."
2349 DATA "richtet sich nach Angebot und Nachfrage, aber der Staat sorgt fuer einen sozialen Ausgleich."
2350 DATA "In welcher Besatzungszone wurde die DDR gegruendet? In der ...",4
2351 DATA "amerikanischen Besatzungszone."
2352 DATA "franzoesischen Besatzungszone."
2353 DATA "britischen Besatzungszone."
2354 DATA "sowjetischen Besatzungszone."
2355 DATA "Die Bundesrepublik Deutschland ist ein Gruendungsmitglied ...",3
2356 DATA "des Nordatlantikpakts (NATO)."
2357 DATA "der Vereinten Nationen (VN)."
2358 DATA "der Europaeischen Union (EU)."
2359 DATA "des Warschauer Pakts."
2360 DATA "Wann wurde die DDR gegruendet?",2
2361 DATA "1947"
2362 DATA "1949"
2363 DATA "1953"
2364 DATA "1956"
2365 DATA "Wie viele Besatzungszonen gab es in Deutschland nach dem Zweiten Weltkrieg?",2
2366 DATA "3"
2367 DATA "4"
2368 DATA "5"
2369 DATA "6"
2370 DATA "Wie waren die Besatzungszonen Deutschlands nach 1945 verteilt?",3
2371 DATA "1=Grossbritannien, 2=Sowjetunion, 3=Frankreich, 4=USA"
2372 DATA "1=Sowjetunion, 2=Grossbritannien, 3=USA, 4=Frankreich"
2373 DATA "1=Grossbritannien, 2=Sowjetunion, 3=USA, 4=Frankreich"
2374 DATA "1=Grossbritannien, 2=USA, 3=Sowjetunion, 4=Frankreich"
2375 DATA "Welche deutsche Stadt wurde nach dem Zweiten Weltkrieg in vier Sektoren aufgeteilt?",2
2376 DATA "Muenchen"
2377 DATA "Berlin"
2378 DATA "Dresden"
2379 DATA "Frankfurt/Oder"
2380 DATA "Vom Juni 1948 bis zum Mai 1949 wurden die Buerger und Buergerinnen von West-Berlin durch eine Luftbruecke versorgt. Welcher Umstand war dafuer verantwortlich?",4
2381 DATA "Fuer Frankreich war eine Versorgung der West-Berliner Bevoelkerung mit dem Flugzeug kostenguenstiger."
2382 DATA "Die amerikanischen Soldaten / Soldatinnen hatten beim Landtransport Angst vor Ueberfaellen."
2383 DATA "Fuer Grossbritannien war die Versorgung ueber die Luftbruecke schneller."
2384 DATA "Die Sowjetunion unterbrach den gesamten Verkehr auf dem Landwege."
2385 DATA "Wie endete der Zweite Weltkrieg in Europa offiziell?",2
2386 DATA "mit dem Tod Adolf Hitlers"
2387 DATA "durch die bedingungslose Kapitulation Deutschlands"
2388 DATA "mit dem Rueckzug der Deutschen aus den besetzten Gebieten"
2389 DATA "durch eine Revolution in Deutschland"
2390 DATA "Der erste Bundeskanzler der Bundesrepublik Deutschland war ...",3
2391 DATA "Ludwig Erhard."
2392 DATA "Willy Brandt."
2393 DATA "Konrad Adenauer."
2394 DATA "Gerhard Schroeder."
2395 DATA "Was wollte Willy Brandt mit seinem Kniefall 1970 im ehemaligen juedischen Ghetto in Warschau ausdruecken?",2
2396 DATA "Er hat sich den ehemaligen Alliierten unterworfen."
2397 DATA "Er bat Polen und die polnischen Juden um Vergebung."
2398 DATA "Er zeigte seine Demut vor dem Warschauer Pakt."
2399 DATA "Er sprach ein Gebet am Grab des Unbekannten Soldaten."
2400 DATA "Wie heisst das juedische Gebetshaus?",3
2401 DATA "Basilika"
2402 DATA "Moschee"
2403 DATA "Synagoge"
2404 DATA "Kirche"
2405 DATA "Wann war in der Bundesrepublik Deutschland das 'Wirtschaftswunder'?",2
2406 DATA "40er Jahre"
2407 DATA "50er Jahre"
2408 DATA "70er Jahre"
2409 DATA "80er Jahre"
2410 DATA "Auf welcher rechtlichen Grundlage wurde der Staat Israel gegruendet?",1
2411 DATA "eine Resolution der Vereinten Nationen"
2412 DATA "ein Beschluss des Zionistenkongresses"
2413 DATA "ein Vorschlag der Bundesregierung"
2414 DATA "ein Vorschlag der UdSSR"
2415 DATA "Wofuer stand der Ausdruck 'Eiserner Vorhang'? Fuer die Abschottung ...",1
2416 DATA "des Warschauer Pakts gegen den Westen."
2417 DATA "Norddeutschlands gegen Sueddeutschland."
2418 DATA "Nazi-Deutschlands gegen die Alliierten."
2419 DATA "Europas gegen die USA."
2420 DATA "Im Jahr 1953 gab es in der DDR einen Aufstand, an den lange Zeit in der Bundesrepublik Deutschland ein Feiertag erinnerte. Wann war das?",2
2421 DATA "1. Mai"
2422 DATA "17. Juni"
2423 DATA "20. Juli"
2424 DATA "9. November"
2425 DATA "Welcher deutsche Staat hatte eine schwarz-rot-goldene Flagge mit Hammer, Zirkel und Aehrenkranz?",3
2426 DATA "Preussen"
2427 DATA "Bundesrepublik Deutschland"
2428 DATA "DDR"
2429 DATA "'Drittes Reich'"
2430 DATA "In welchem Jahr wurde die Mauer in Berlin gebaut?",4
2431 DATA "1953"
2432 DATA "1956"
2433 DATA "1959"
2434 DATA "1961"
2435 DATA "Wann baute die DDR die Mauer in Berlin?",3
2436 DATA "1919"
2437 DATA "1933"
2438 DATA "1961"
2439 DATA "1990"
2440 DATA "Was bedeutet die Abkuerzung DDR?",4
2441 DATA "Dritter Deutscher Rundfunk"
2442 DATA "Die Deutsche Republik"
2443 DATA "Dritte Deutsche Republik"
2444 DATA "Deutsche Demokratische Republik"
2445 DATA "Wann wurde die Mauer in Berlin fuer alle geoeffnet?",2
2446 DATA "1987"
2447 DATA "1989"
2448 DATA "1992"
2449 DATA "1995"
2450 DATA "Welches heutige deutsche Bundesland gehoerte frueher zum Gebiet der DDR?",1
2451 DATA "Brandenburg"
2452 DATA "Bayern"
2453 DATA "Saarland"
2454 DATA "Hessen"
2455 DATA "Von 1961 bis 1989 war Berlin ...",3
2456 DATA "ohne Buergermeister."
2457 DATA "ein eigener Staat."
2458 DATA "durch eine Mauer geteilt."
2459 DATA "nur mit dem Flugzeug erreichbar."
2460 DATA "Am 3. Oktober feiert man in Deutschland den Tag der Deutschen ...",1
2461 DATA "Einheit."
2462 DATA "Nation."
2463 DATA "Bundeslaender."
2464 DATA "Staedte."
2465 DATA "Welches heutige deutsche Bundesland gehoerte frueher zum Gebiet der DDR?",2
2466 DATA "Hessen"
2467 DATA "Sachsen-Anhalt"
2468 DATA "Nordrhein-Westfalen"
2469 DATA "Saarland"
2470 DATA "Warum nennt man die Zeit im Herbst 1989 in der DDR 'Die Wende'? In dieser Zeit veraenderte sich die DDR politisch ...",1
2471 DATA "von einer Diktatur zur Demokratie."
2472 DATA "von einer liberalen Marktwirtschaft zum Sozialismus."
2473 DATA "von einer Monarchie zur Sozialdemokratie."
2474 DATA "von einem religioesen Staat zu einem kommunistischen Staat."
2475 DATA "Welches heutige deutsche Bundesland gehoerte frueher zum Gebiet der DDR?",1
2476 DATA "Thueringen"
2477 DATA "Hessen"
2478 DATA "Bayern"
2479 DATA "Bremen"
2480 DATA "Welches heutige deutsche Bundesland gehoerte frueher zum Gebiet der DDR?",3
2481 DATA "Bayern"
2482 DATA "Niedersachsen"
2483 DATA "Sachsen"
2484 DATA "Baden-Wuerttemberg"
2485 DATA "Mit der Abkuerzung 'Stasi' meinte man in der DDR ...",2
2486 DATA "das Parlament."
2487 DATA "das Ministerium fuer Staatssicherheit."
2488 DATA "eine regierende Partei."
2489 DATA "das Ministerium fuer Volksbildung."
2490 DATA "Welches heutige deutsche Bundesland gehoerte frueher zum Gebiet der DDR?",3
2491 DATA "Hessen"
2492 DATA "Schleswig-Holstein"
2493 DATA "Mecklenburg-Vorpommern"
2494 DATA "Saarland"
