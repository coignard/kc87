10 REM LID-TRAINER TEIL 1 VON 3
12 REM COPYRIGHT (C) 2026 R. COIGNARD
14 CLEAR 4000:WINDOW
16 DIM O$(4):DIM V(4):DIM OZ(4):DIM P(99):DG$="1234"
18 GOTO 100
100 BORDER 5:PAPER 5:INK 8:CLS
102 PRINT AT(2,10);"LEBEN IN DEUTSCHLAND"
104 PRINT AT(4,12);"PRUEFUNGSTRAINER"
106 PRINT AT(6,8);"TEIL 1 VON 3   99 FRAGEN"
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
212 HE$="TEIL 1  FRAGE "+MID$(STR$(QQ),2)+"/99  RICHTIG ":HD$=HE$+MID$(STR$(S),2)
214 GOSUB 1100
216 GOSUB 1400
218 GOSUB 1600
220 IF AC=2 THEN 100
222 NEXT QQ
224 GOTO 100
300 S=0
302 BORDER 5:PAPER 5:INK 8:CLS
304 PRINT AT(2,4);"TESTMODUS - TEIL 1"
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
342 HE$="TEIL 1  FRAGE "+MID$(STR$(QI),2)+"/"+MID$(STR$(TC),2)+"  RICHTIG ":HD$=HE$+MID$(STR$(S),2)
344 GOSUB 1100
346 GOSUB 1400
348 GOSUB 1600
350 IF AC=2 THEN 100
352 NEXT QI
354 BORDER 5:PAPER 5:INK 8:CLS
356 PRINT AT(4,6);"ERGEBNIS - TEIL 1"
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
2000 DATA "In Deutschland duerfen Menschen offen etwas gegen die Regierung sagen, weil ...",4
2001 DATA "hier Religionsfreiheit gilt."
2002 DATA "die Menschen Steuern zahlen."
2003 DATA "die Menschen das Wahlrecht haben."
2004 DATA "hier Meinungsfreiheit gilt."
2005 DATA "In Deutschland koennen Eltern bis zum 14. Lebensjahr ihres Kindes entscheiden, ob es in der Schule am ...",2
2006 DATA "Geschichtsunterricht teilnimmt."
2007 DATA "Religionsunterricht teilnimmt."
2008 DATA "Politikunterricht teilnimmt."
2009 DATA "Sprachunterricht teilnimmt."
2010 DATA "Deutschland ist ein Rechtsstaat. Was ist damit gemeint?",1
2011 DATA "Alle Einwohner / Einwohnerinnen und der Staat muessen sich an die Gesetze halten."
2012 DATA "Der Staat muss sich nicht an die Gesetze halten."
2013 DATA "Nur Deutsche muessen die Gesetze befolgen."
2014 DATA "Die Gerichte machen die Gesetze."
2015 DATA "Welches Recht gehoert zu den Grundrechten in Deutschland?",3
2016 DATA "Waffenbesitz"
2017 DATA "Faustrecht"
2018 DATA "Meinungsfreiheit"
2019 DATA "Selbstjustiz"
2020 DATA "Wahlen in Deutschland sind frei. Was bedeutet das?",2
2021 DATA "Man darf Geld annehmen, wenn man dafuer einen bestimmten Kandidaten / eine bestimmte Kandidatin waehlt."
2022 DATA "Der Waehler darf bei der Wahl weder beeinflusst noch zu einer bestimmten Stimmabgabe gezwungen werden und keine Nachteile durch die Wahl haben."
2023 DATA "Nur Personen, die noch nie im Gefaengnis waren, duerfen waehlen."
2024 DATA "Alle wahlberechtigten Personen muessen waehlen."
2025 DATA "Wie heisst die deutsche Verfassung?",4
2026 DATA "Volksgesetz"
2027 DATA "Bundesgesetz"
2028 DATA "Deutsches Gesetz"
2029 DATA "Grundgesetz"
2030 DATA "Welches Recht gehoert zu den Grundrechten, die nach der deutschen Verfassung garantiert werden? Das Recht auf ...",1
2031 DATA "Glaubens- und Gewissensfreiheit"
2032 DATA "Unterhaltung"
2033 DATA "Arbeit"
2034 DATA "Wohnung"
2035 DATA "Was steht nicht im Grundgesetz von Deutschland?",2
2036 DATA "Die Wuerde des Menschen ist unantastbar."
2037 DATA "Alle sollen gleich viel Geld haben."
2038 DATA "Jeder Mensch darf seine Meinung sagen."
2039 DATA "Alle sind vor dem Gesetz gleich."
2040 DATA "Welches Grundrecht gilt in Deutschland nur fuer Auslaender / Auslaenderinnen? Das Grundrecht auf ...",3
2041 DATA "Schutz der Familie"
2042 DATA "Menschenwuerde"
2043 DATA "Asyl"
2044 DATA "Meinungsfreiheit"
2045 DATA "Was ist mit dem deutschen Grundgesetz vereinbar?",4
2046 DATA "die Pruegelstrafe"
2047 DATA "die Folter"
2048 DATA "die Todesstrafe"
2049 DATA "die Geldstrafe"
2050 DATA "Wie wird die Verfassung der Bundesrepublik Deutschland genannt?",1
2051 DATA "Grundgesetz"
2052 DATA "Bundesverfassung"
2053 DATA "Gesetzbuch"
2054 DATA "Verfassungsvertrag"
2055 DATA "Eine Partei im Deutschen Bundestag will die Pressefreiheit abschaffen. Ist das moeglich?",3
2056 DATA "Ja, wenn mehr als die Haelfte der Abgeordneten im Bundestag dafuer sind."
2057 DATA "Ja, aber dazu muessen zwei Drittel der Abgeordneten im Bundestag dafuer sein."
2058 DATA "Nein, denn die Pressefreiheit ist ein Grundrecht. Sie kann nicht abgeschafft werden."
2059 DATA "Nein, denn nur der Bundesrat kann die Pressefreiheit abschaffen."
2060 DATA "Im Parlament steht der Begriff 'Opposition' fuer ...",2
2061 DATA "die regierenden Parteien."
2062 DATA "alle Abgeordneten, die nicht zu der Regierungspartei/den Regierungsparteien gehoeren."
2063 DATA "die Fraktion mit den meisten Abgeordneten."
2064 DATA "alle Parteien, die bei der letzten Wahl die 5 %-Huerde erreichen konnten."
2065 DATA "Meinungsfreiheit in Deutschland heisst zum Beispiel, dass ich ...",2
2066 DATA "Passanten auf der Strasse beschimpfen darf."
2067 DATA "meine Meinung im Internet aeussern kann."
2068 DATA "Nazi-, Hamas- oder Islamischer Staat-Symbole oeffentlich tragen darf."
2069 DATA "meine Meinung nur dann aeussern darf, solange ich der Regierung nicht widerspreche."
2070 DATA "Was verbietet das deutsche Grundgesetz?",2
2071 DATA "Militaerdienst"
2072 DATA "Zwangsarbeit"
2073 DATA "freie Berufswahl"
2074 DATA "Arbeit im Ausland"
2075 DATA "Wann ist die Meinungsfreiheit in Deutschland eingeschraenkt?",1
2076 DATA "bei der oeffentlichen Verbreitung falscher Behauptungen ueber einzelne Personen"
2077 DATA "bei Meinungsaeusserungen ueber die Bundesregierung"
2078 DATA "bei Diskussionen ueber Religionen"
2079 DATA "bei Kritik am Staat"
2080 DATA "Die deutschen Gesetze verbieten ...",4
2081 DATA "Meinungsfreiheit der Einwohner und Einwohnerinnen."
2082 DATA "Petitionen der Buerger und Buergerinnen."
2083 DATA "Versammlungsfreiheit der Einwohner und Einwohnerinnen."
2084 DATA "Ungleichbehandlung der Buerger und Buergerinnen durch den Staat."
2085 DATA "Welches Grundrecht ist in Artikel 1 des Grundgesetzes der Bundesrepublik Deutschland garantiert?",1
2086 DATA "die Unantastbarkeit der Menschenwuerde"
2087 DATA "das Recht auf Leben"
2088 DATA "Religionsfreiheit"
2089 DATA "Meinungsfreiheit"
2090 DATA "Was versteht man unter dem Recht der 'Freizuegigkeit' in Deutschland?",1
2091 DATA "Man darf sich seinen Wohnort selbst aussuchen."
2092 DATA "Man kann seinen Beruf wechseln."
2093 DATA "Man darf sich fuer eine andere Religion entscheiden."
2094 DATA "Man darf sich in der Oeffentlichkeit nur leicht bekleidet bewegen."
2095 DATA "Eine Partei in Deutschland verfolgt das Ziel, eine Diktatur zu errichten. Sie ist dann ...",4
2096 DATA "tolerant."
2097 DATA "rechtsstaatlich orientiert."
2098 DATA "gesetzestreu."
2099 DATA "verfassungswidrig."
2100 DATA "Was fuer eine Staatsform hat Deutschland?",3
2101 DATA "Monarchie"
2102 DATA "Diktatur"
2103 DATA "Republik"
2104 DATA "Fuerstentum"
2105 DATA "In Deutschland sind die meisten Erwerbstaetigen ...",1
2106 DATA "bei einer Firma oder Behoerde beschaeftigt."
2107 DATA "in kleinen Familienunternehmen beschaeftigt."
2108 DATA "ehrenamtlich fuer ein Bundesland taetig."
2109 DATA "selbstaendig mit einer eigenen Firma taetig."
2110 DATA "Wie viele Bundeslaender hat die Bundesrepublik Deutschland?",3
2111 DATA "14"
2112 DATA "15"
2113 DATA "16"
2114 DATA "17"
2115 DATA "Was ist kein Bundesland der Bundesrepublik Deutschland?",2
2116 DATA "Nordrhein-Westfalen"
2117 DATA "Elsass-Lothringen"
2118 DATA "Mecklenburg-Vorpommern"
2119 DATA "Sachsen-Anhalt"
2120 DATA "Deutschland ist ...",2
2121 DATA "eine kommunistische Republik."
2122 DATA "ein demokratischer und sozialer Bundesstaat."
2123 DATA "eine kapitalistische und soziale Monarchie."
2124 DATA "ein sozialer und sozialistischer Bundesstaat."
2125 DATA "Deutschland ist ...",2
2126 DATA "ein sozialistischer Staat."
2127 DATA "ein Bundesstaat."
2128 DATA "eine Diktatur."
2129 DATA "eine Monarchie."
2130 DATA "Wer waehlt in Deutschland die Abgeordneten zum Bundestag?",3
2131 DATA "das Militaer"
2132 DATA "die Wirtschaft"
2133 DATA "das wahlberechtigte Volk"
2134 DATA "die Verwaltung"
2135 DATA "Welches Tier ist das Wappentier der Bundesrepublik Deutschland?",2
2136 DATA "Loewe"
2137 DATA "Adler"
2138 DATA "Baer"
2139 DATA "Pferd"
2140 DATA "Was ist kein Merkmal unserer Demokratie?",2
2141 DATA "regelmaessige Wahlen"
2142 DATA "Pressezensur"
2143 DATA "Meinungsfreiheit"
2144 DATA "verschiedene Parteien"
2145 DATA "Die Zusammenarbeit von Parteien zur Bildung einer Regierung nennt man in Deutschland ...",2
2146 DATA "Einheit."
2147 DATA "Koalition."
2148 DATA "Ministerium."
2149 DATA "Fraktion."
2150 DATA "Was ist keine staatliche Gewalt in Deutschland?",3
2151 DATA "Gesetzgebung"
2152 DATA "Regierung"
2153 DATA "Presse"
2154 DATA "Rechtsprechung"
2155 DATA "Welche Aussage ist richtig? In Deutschland ...",1
2156 DATA "sind Staat und Religionsgemeinschaften voneinander getrennt."
2157 DATA "bilden die Religionsgemeinschaften den Staat."
2158 DATA "ist der Staat abhaengig von den Religionsgemeinschaften."
2159 DATA "bilden Staat und Religionsgemeinschaften eine Einheit."
2160 DATA "Was ist Deutschland nicht?",3
2161 DATA "eine Demokratie"
2162 DATA "ein Rechtsstaat"
2163 DATA "eine Monarchie"
2164 DATA "ein Sozialstaat"
2165 DATA "Womit finanziert der deutsche Staat die Sozialversicherung?",2
2166 DATA "Kirchensteuern"
2167 DATA "Sozialabgaben"
2168 DATA "Spendengeldern"
2169 DATA "Vereinsbeitraegen"
2170 DATA "Welche Massnahme schafft in Deutschland soziale Sicherheit?",1
2171 DATA "die Krankenversicherung"
2172 DATA "die Autoversicherung"
2173 DATA "die Gebaeudeversicherung"
2174 DATA "die Haftpflichtversicherung"
2175 DATA "Wie werden die Regierungschefs / Regierungschefinnen der meisten Bundeslaender in Deutschland genannt?",4
2176 DATA "Erster Minister / Erste Ministerin"
2177 DATA "Premierminister / Premierministerin"
2178 DATA "Senator / Senatorin"
2179 DATA "Ministerpraesident / Ministerpraesidentin"
2180 DATA "Die Bundesrepublik Deutschland ist ein demokratischer und sozialer ...",2
2181 DATA "Staatenverbund."
2182 DATA "Bundesstaat."
2183 DATA "Staatenbund."
2184 DATA "Zentralstaat."
2185 DATA "Was hat jedes deutsche Bundesland?",4
2186 DATA "einen eigenen Aussenminister / eine eigene Aussenministerin"
2187 DATA "eine eigene Waehrung"
2188 DATA "eine eigene Armee"
2189 DATA "eine eigene Regierung"
2190 DATA "Mit welchen Worten beginnt die deutsche Nationalhymne?",2
2191 DATA "Voelker, hoert die Signale ..."
2192 DATA "Einigkeit und Recht und Freiheit ..."
2193 DATA "Freude schoener Goetterfunken ..."
2194 DATA "Deutschland einig Vaterland ..."
2195 DATA "Warum gibt es in einer Demokratie mehr als eine Partei?",1
2196 DATA "weil dadurch die unterschiedlichen Meinungen der Buerger und Buergerinnen vertreten werden"
2197 DATA "damit Bestechung in der Politik begrenzt wird"
2198 DATA "um politische Demonstrationen zu verhindern"
2199 DATA "um wirtschaftlichen Wettbewerb anzuregen"
2200 DATA "Wer beschliesst in Deutschland ein neues Gesetz?",2
2201 DATA "die Regierung"
2202 DATA "das Parlament"
2203 DATA "die Gerichte"
2204 DATA "die Polizei"
2205 DATA "Wann kann in Deutschland eine Partei verboten werden?",2
2206 DATA "wenn ihr Wahlkampf zu teuer ist"
2207 DATA "wenn sie gegen die Verfassung kaempft"
2208 DATA "wenn sie Kritik am Staatsoberhaupt aeussert"
2209 DATA "wenn ihr Programm eine neue Richtung vorschlaegt"
2210 DATA "Wen kann man als Buerger / Buergerin in Deutschland nicht direkt waehlen?",2
2211 DATA "Abgeordnete des EU-Parlaments"
2212 DATA "den Bundespraesidenten / die Bundespraesidentin"
2213 DATA "Landtagsabgeordnete"
2214 DATA "Bundestagsabgeordnete"
2215 DATA "Zu welcher Versicherung gehoert die Pflegeversicherung?",1
2216 DATA "Sozialversicherung"
2217 DATA "Unfallversicherung"
2218 DATA "Hausratversicherung"
2219 DATA "Haftpflicht- und Feuerversicherung"
2220 DATA "Der deutsche Staat hat viele Aufgaben. Welche Aufgabe gehoert dazu?",1
2221 DATA "Er baut Strassen und Schulen."
2222 DATA "Er verkauft Lebensmittel und Kleidung."
2223 DATA "Er versorgt alle Einwohner und Einwohnerinnen kostenlos mit Zeitungen."
2224 DATA "Er produziert Autos und Busse."
2225 DATA "Der deutsche Staat hat viele Aufgaben. Welche Aufgabe gehoert nicht dazu?",1
2226 DATA "Er bezahlt fuer alle Staatsangehoerigen Urlaubsreisen."
2227 DATA "Er zahlt Kindergeld."
2228 DATA "Er unterstuetzt Museen."
2229 DATA "Er foerdert Sportler und Sportlerinnen."
2230 DATA "Welches Organ gehoert nicht zu den Verfassungsorganen Deutschlands?",3
2231 DATA "der Bundesrat"
2232 DATA "der Bundespraesident / die Bundespraesidentin"
2233 DATA "die Buergerversammlung"
2234 DATA "die Regierung"
2235 DATA "Wer bestimmt in Deutschland die Schulpolitik?",2
2236 DATA "die Lehrer und Lehrerinnen"
2237 DATA "die Bundeslaender"
2238 DATA "das Familienministerium"
2239 DATA "die Universitaeten"
2240 DATA "Die Wirtschaftsform in Deutschland nennt man ...",2
2241 DATA "freie Zentralwirtschaft."
2242 DATA "soziale Marktwirtschaft."
2243 DATA "gelenkte Zentralwirtschaft."
2244 DATA "Planwirtschaft."
2245 DATA "Zu einem demokratischen Rechtsstaat gehoert es nicht, dass ...",3
2246 DATA "Menschen sich kritisch ueber die Regierung aeussern koennen."
2247 DATA "Buerger friedlich demonstrieren gehen duerfen."
2248 DATA "Menschen von einer Privatpolizei ohne Grund verhaftet werden."
2249 DATA "jemand ein Verbrechen begeht und deshalb verhaftet wird."
2250 DATA "Was bedeutet 'Volkssouveraenitaet'? Alle Staatsgewalt geht vom ...",1
2251 DATA "Volke aus."
2252 DATA "Bundestag aus."
2253 DATA "preussischen Koenig aus."
2254 DATA "Bundesverfassungsgericht aus."
2255 DATA "Was bedeutet 'Rechtsstaat' in Deutschland?",4
2256 DATA "Der Staat hat Recht."
2257 DATA "Es gibt nur rechte Parteien."
2258 DATA "Die Buerger und Buergerinnen entscheiden ueber Gesetze."
2259 DATA "Der Staat muss die Gesetze einhalten."
2260 DATA "Was ist keine staatliche Gewalt in Deutschland?",4
2261 DATA "Legislative"
2262 DATA "Judikative"
2263 DATA "Exekutive"
2264 DATA "Direktive"
2265 DATA "Was zeigt dieses Bild?",1
2266 DATA "den Bundestagssitz in Berlin"
2267 DATA "das Bundesverfassungsgericht in Karlsruhe"
2268 DATA "das Bundesratsgebaeude in Berlin"
2269 DATA "das Bundeskanzleramt in Berlin"
2270 DATA "Welches Amt gehoert in Deutschland zur Gemeindeverwaltung?",2
2271 DATA "Pfarramt"
2272 DATA "Ordnungsamt"
2273 DATA "Finanzamt"
2274 DATA "Auswaertiges Amt"
2275 DATA "Wer wird meistens zum Praesidenten / zur Praesidentin des Deutschen Bundestages gewaehlt?",3
2276 DATA "der / die aelteste Abgeordnete im Parlament"
2277 DATA "der Ministerpraesident / die Ministerpraesidentin des groessten Bundeslandes"
2278 DATA "ein Abgeordneter / eine Abgeordnete der staerksten Fraktion"
2279 DATA "ein ehemaliger Bundeskanzler / eine ehemalige Bundeskanzlerin"
2280 DATA "Wer ernennt in Deutschland die Minister / die Ministerinnen der Bundesregierung?",2
2281 DATA "der Praesident / die Praesidentin des Bundesverfassungsgerichtes"
2282 DATA "der Bundespraesident / die Bundespraesidentin"
2283 DATA "der Bundesratspraesident / die Bundesratspraesidentin"
2284 DATA "der Bundestagspraesident / die Bundestagspraesidentin"
2285 DATA "Vor wie vielen Jahren gab es erstmals eine juedische Gemeinde auf dem Gebiet des heutigen Deutschlands?",4
2286 DATA "vor etwa 300 Jahren"
2287 DATA "vor etwa 700 Jahren"
2288 DATA "vor etwa 1150 Jahren"
2289 DATA "vor etwa 1700 Jahren"
2290 DATA "In Deutschland gehoeren der Bundestag und der Bundesrat zur ...",2
2291 DATA "Exekutive."
2292 DATA "Legislative."
2293 DATA "Direktive."
2294 DATA "Judikative."
2295 DATA "Was bedeutet 'Volkssouveraenitaet'?",4
2296 DATA "Der Koenig / die Koenigin herrscht ueber das Volk."
2297 DATA "Das Bundesverfassungsgericht steht ueber der Verfassung."
2298 DATA "Die Interessenverbaende ueben die Souveraenitaet zusammen mit der Regierung aus."
2299 DATA "Die Staatsgewalt geht vom Volke aus."
2300 DATA "Wenn das Parlament eines deutschen Bundeslandes gewaehlt wird, nennt man das ...",2
2301 DATA "Kommunalwahl."
2302 DATA "Landtagswahl."
2303 DATA "Europawahl."
2304 DATA "Bundestagswahl."
2305 DATA "Was gehoert in Deutschland nicht zur Exekutive?",2
2306 DATA "die Polizei"
2307 DATA "die Gerichte"
2308 DATA "das Finanzamt"
2309 DATA "die Ministerien"
2310 DATA "Die Bundesrepublik Deutschland ist heute gegliedert in ...",4
2311 DATA "vier Besatzungszonen."
2312 DATA "einen Oststaat und einen Weststaat."
2313 DATA "16 Kantone."
2314 DATA "Bund, Laender und Kommunen."
2315 DATA "Es gehoert nicht zu den Aufgaben des Deutschen Bundestages, ...",4
2316 DATA "Gesetze zu entwerfen."
2317 DATA "die Bundesregierung zu kontrollieren."
2318 DATA "den Bundeskanzler / die Bundeskanzlerin zu waehlen."
2319 DATA "das Bundeskabinett zu bilden."
2320 DATA "Welche Staedte haben die groessten juedischen Gemeinden in Deutschland?",1
2321 DATA "Berlin und Muenchen"
2322 DATA "Hamburg und Essen"
2323 DATA "Nuernberg und Stuttgart"
2324 DATA "Worms und Speyer"
2325 DATA "Was ist in Deutschland vor allem eine Aufgabe der Bundeslaender?",4
2326 DATA "Verteidigungspolitik"
2327 DATA "Aussenpolitik"
2328 DATA "Wirtschaftspolitik"
2329 DATA "Schulpolitik"
2330 DATA "Warum kontrolliert der Staat in Deutschland das Schulwesen?",4
2331 DATA "weil es in Deutschland nur staatliche Schulen gibt"
2332 DATA "weil alle Schueler und Schuelerinnen einen Schulabschluss haben muessen"
2333 DATA "weil es in den Bundeslaendern verschiedene Schulen gibt"
2334 DATA "weil es nach dem Grundgesetz seine Aufgabe ist"
2335 DATA "Die Bundesrepublik Deutschland hat einen dreistufigen Verwaltungsaufbau. Wie heisst die unterste politische Stufe?",3
2336 DATA "Stadtraete"
2337 DATA "Landraete"
2338 DATA "Gemeinden"
2339 DATA "Bezirksaemter"
2340 DATA "Der deutsche Bundespraesident Gustav Heinemann gibt Helmut Schmidt 1974 die Ernennungsurkunde zum deutschen Bundeskanzler. Was gehoert zu den Aufgaben des deutschen Bundespraesidenten / der deutschen Bundespraesidentin?",4
2341 DATA "Er / Sie fuehrt die Regierungsgeschaefte."
2342 DATA "Er / Sie kontrolliert die Regierungspartei."
2343 DATA "Er / Sie waehlt die Minister / Ministerinnen aus."
2344 DATA "Er / Sie schlaegt den Kanzler / die Kanzlerin zur Wahl vor."
2345 DATA "Wo haelt sich der deutsche Bundeskanzler / die deutsche Bundeskanzlerin am haeufigsten auf? Am haeufigsten ist er / sie ...",2
2346 DATA "in Bonn, weil sich dort das Bundeskanzleramt und der Bundestag befinden."
2347 DATA "in Berlin, weil sich dort das Bundeskanzleramt und der Bundestag befinden."
2348 DATA "auf Schloss Meseberg, dem Gaestehaus der Bundesregierung, um Staatsgaeste zu empfangen."
2349 DATA "auf Schloss Bellevue, dem Amtssitz des Bundespraesidenten / der Bundespraesidentin, um Staatsgaeste zu empfangen."
2350 DATA "Wie heisst der jetzige Bundeskanzler / die jetzige Bundeskanzlerin von Deutschland?",4
2351 DATA "Gerhard Schroeder"
2352 DATA "Angela Merkel"
2353 DATA "Ursula von der Leyen"
2354 DATA "Friedrich Merz"
2355 DATA "Die beiden groessten Fraktionen im Deutschen Bundestag heissen zurzeit ...",1
2356 DATA "CDU/CSU und AfD."
2357 DATA "Die Linke und Buendnis 90/Die Gruenen."
2358 DATA "Buendnis 90/Die Gruenen und SPD."
2359 DATA "Die Linke und CDU/CSU."
2360 DATA "Wie heisst das Parlament fuer ganz Deutschland?",3
2361 DATA "Bundesversammlung"
2362 DATA "Volkskammer"
2363 DATA "Bundestag"
2364 DATA "Bundesgerichtshof"
2365 DATA "Wie heisst Deutschlands heutiges Staatsoberhaupt?",1
2366 DATA "Frank-Walter Steinmeier"
2367 DATA "Baerbel Bas"
2368 DATA "Bodo Ramelow"
2369 DATA "Joachim Gauck"
2370 DATA "Was bedeutet die Abkuerzung CDU in Deutschland?",4
2371 DATA "Christliche Deutsche Union"
2372 DATA "Club Deutscher Unternehmer"
2373 DATA "Christlicher Deutscher Umweltschutz"
2374 DATA "Christlich Demokratische Union"
2375 DATA "Was ist die Bundeswehr?",4
2376 DATA "die deutsche Polizei"
2377 DATA "ein deutscher Hafen"
2378 DATA "eine deutsche Buergerinitiative"
2379 DATA "die deutsche Armee"
2380 DATA "Was bedeutet die Abkuerzung SPD?",3
2381 DATA "Sozialistische Partei Deutschlands"
2382 DATA "Sozialpolitische Partei Deutschlands"
2383 DATA "Sozialdemokratische Partei Deutschlands"
2384 DATA "Sozialgerechte Partei Deutschlands"
2385 DATA "Was bedeutet die Abkuerzung FDP in Deutschland?",4
2386 DATA "Friedliche Demonstrative Partei"
2387 DATA "Freie Deutschland Partei"
2388 DATA "Fuehrende Demokratische Partei"
2389 DATA "Freie Demokratische Partei"
2390 DATA "Welches Gericht in Deutschland ist zustaendig fuer die Auslegung des Grundgesetzes?",3
2391 DATA "Oberlandesgericht"
2392 DATA "Amtsgericht"
2393 DATA "Bundesverfassungsgericht"
2394 DATA "Verwaltungsgericht"
2395 DATA "Wer waehlt den Bundeskanzler / die Bundeskanzlerin in Deutschland?",4
2396 DATA "der Bundesrat"
2397 DATA "die Bundesversammlung"
2398 DATA "das Volk"
2399 DATA "der Bundestag"
2400 DATA "Wer leitet das deutsche Bundeskabinett?",3
2401 DATA "der Bundestagspraesident / die Bundestagspraesidentin"
2402 DATA "der Bundespraesident / die Bundespraesidentin"
2403 DATA "der Bundeskanzler / die Bundeskanzlerin"
2404 DATA "der Bundesratspraesident / die Bundesratspraesidentin"
2405 DATA "Wer waehlt den deutschen Bundeskanzler / die deutsche Bundeskanzlerin?",3
2406 DATA "das Volk"
2407 DATA "die Bundesversammlung"
2408 DATA "der Bundestag"
2409 DATA "die Bundesregierung"
2410 DATA "Welche Hauptaufgabe hat der deutsche Bundespraesident / die deutsche Bundespraesidentin? Er / Sie ...",3
2411 DATA "regiert das Land."
2412 DATA "entwirft die Gesetze."
2413 DATA "repraesentiert das Land."
2414 DATA "ueberwacht die Einhaltung der Gesetze."
2415 DATA "Wer bildet den deutschen Bundesrat?",3
2416 DATA "die Abgeordneten des Bundestages"
2417 DATA "die Minister und Ministerinnen der Bundesregierung"
2418 DATA "die Regierungsvertreter der Bundeslaender"
2419 DATA "die Parteimitglieder"
2420 DATA "Wer waehlt in Deutschland den Bundespraesidenten / die Bundespraesidentin?",1
2421 DATA "die Bundesversammlung"
2422 DATA "der Bundesrat"
2423 DATA "das Bundesparlament"
2424 DATA "das Bundesverfassungsgericht"
2425 DATA "Wer ist das Staatsoberhaupt der Bundesrepublik Deutschland?",2
2426 DATA "der Bundeskanzler / die Bundeskanzlerin"
2427 DATA "der Bundespraesident / die Bundespraesidentin"
2428 DATA "der Bundesratspraesident / die Bundesratspraesidentin"
2429 DATA "der Bundestagspraesident / die Bundestagspraesidentin"
2430 DATA "Die parlamentarische Opposition im Deutschen Bundestag ...",1
2431 DATA "kontrolliert die Regierung."
2432 DATA "entscheidet, wer Bundesminister / Bundesministerin wird."
2433 DATA "bestimmt, wer im Bundesrat sitzt."
2434 DATA "schlaegt die Regierungschefs / Regierungschefinnen der Laender vor."
2435 DATA "Wie nennt man in Deutschland die Vereinigung von Abgeordneten einer Partei im Parlament?",3
2436 DATA "Verband"
2437 DATA "Aeltestenrat"
2438 DATA "Fraktion"
2439 DATA "Opposition"
2440 DATA "Die deutschen Bundeslaender wirken an der Gesetzgebung des Bundes mit durch ...",1
2441 DATA "den Bundesrat."
2442 DATA "die Bundesversammlung."
2443 DATA "den Bundestag."
2444 DATA "die Bundesregierung."
2445 DATA "In Deutschland kann ein Regierungswechsel in einem Bundesland Auswirkungen auf die Bundespolitik haben. Das Regieren wird ...",3
2446 DATA "schwieriger, wenn sich dadurch die Mehrheit im Bundestag aendert."
2447 DATA "leichter, wenn dadurch neue Parteien in den Bundesrat kommen."
2448 DATA "schwieriger, wenn dadurch die Mehrheit im Bundesrat veraendert wird."
2449 DATA "leichter, wenn es sich um ein reiches Bundesland handelt."
2450 DATA "Was bedeutet die Abkuerzung CSU in Deutschland?",4
2451 DATA "Christlich Sichere Union"
2452 DATA "Christlich Sueddeutsche Union"
2453 DATA "Christlich Sozialer Unternehmerverband"
2454 DATA "Christlich Soziale Union"
2455 DATA "Je mehr 'Zweitstimmen' eine Partei bei einer Bundestagswahl bekommt, desto ...",1
2456 DATA "mehr Sitze erhaelt die Partei im Parlament."
2457 DATA "weniger Erststimmen kann sie haben."
2458 DATA "mehr Direktkandidaten der Partei ziehen ins Parlament ein."
2459 DATA "groesser ist das Risiko, eine Koalition bilden zu muessen."
2460 DATA "Ab welchem Alter darf man in Deutschland an der Wahl zum Deutschen Bundestag teilnehmen?",2
2461 DATA "16"
2462 DATA "18"
2463 DATA "21"
2464 DATA "23"
2465 DATA "Was gilt fuer die meisten Kinder in Deutschland?",2
2466 DATA "Wahlpflicht"
2467 DATA "Schulpflicht"
2468 DATA "Schweigepflicht"
2469 DATA "Religionspflicht"
2470 DATA "Wie kann jemand, der den Holocaust leugnet, bestraft werden?",4
2471 DATA "Kuerzung sozialer Leistungen"
2472 DATA "bis zu 100 Sozialstunden"
2473 DATA "gar nicht, Holocaustleugnung ist erlaubt"
2474 DATA "mit Freiheitsstrafe bis zu fuenf Jahren oder mit Geldstrafe"
2475 DATA "Was bezahlt man in Deutschland automatisch, wenn man fest angestellt ist?",1
2476 DATA "Sozialversicherung"
2477 DATA "Sozialhilfe"
2478 DATA "Kindergeld"
2479 DATA "Wohngeld"
2480 DATA "Wenn Abgeordnete im Deutschen Bundestag ihre Fraktion wechseln, ...",1
2481 DATA "kann die Regierung ihre Mehrheit verlieren."
2482 DATA "duerfen sie nicht mehr an den Sitzungen des Parlaments teilnehmen."
2483 DATA "muss der Bundespraesident / die Bundespraesidentin zuvor sein / ihr Einverstaendnis geben."
2484 DATA "duerfen die Waehler / Waehlerinnen dieser Abgeordneten noch einmal waehlen."
2485 DATA "Wer bezahlt in Deutschland die Sozialversicherungen?",1
2486 DATA "Arbeitgeber / Arbeitgeberinnen und Arbeitnehmer / Arbeitnehmerinnen"
2487 DATA "nur Arbeitnehmer / Arbeitnehmerinnen"
2488 DATA "alle Staatsangehoerigen"
2489 DATA "nur Arbeitgeber / Arbeitgeberinnen"
2490 DATA "Was gehoert nicht zur gesetzlichen Sozialversicherung?",2
2491 DATA "die gesetzliche Rentenversicherung"
2492 DATA "die Lebensversicherung"
2493 DATA "die Arbeitslosenversicherung"
2494 DATA "die Pflegeversicherung"
