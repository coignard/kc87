10 REM LID-TRAINER TEIL 3 VON 3
12 REM COPYRIGHT (C) 2026 R. COIGNARD
14 CLEAR 4000:WINDOW
16 DIM O$(4):DIM V(4):DIM OZ(4):DIM P(98):DG$="1234"
18 GOTO 100
100 BORDER 5:PAPER 5:INK 8:CLS
102 PRINT AT(2,10);"LEBEN IN DEUTSCHLAND"
104 PRINT AT(4,12);"PRUEFUNGSTRAINER"
106 PRINT AT(6,8);"TEIL 3 VON 3   98 FRAGEN"
108 PRINT AT(11,6);"1. LERNEN  - ALLE FRAGEN"
110 PRINT AT(13,6);"2. TEST    - MIT ZUFALLSZAHL"
112 PRINT AT(23,5);"COPYRIGHT (C) 2026 R. COIGNARD"
116 K$=INKEY$:IF K$="1" THEN 200
118 IF K$="2" THEN 300
120 GOTO 116
200 S=0
202 FOR I=1 TO 4:V(I)=I:NEXT
204 FOR QQ=1 TO 98
206 KK=QQ:GOSUB 1000
208 CP=0:FOR M=1 TO 4:IF V(M)=C THEN CP=M
210 NEXT M
212 HE$="TEIL 3  FRAGE "+MID$(STR$(QQ),2)+"/98  RICHTIG ":HD$=HE$+MID$(STR$(S),2)
214 GOSUB 1100
216 GOSUB 1400
218 GOSUB 1600
220 IF AC=2 THEN 100
222 NEXT QQ
224 GOTO 100
300 S=0
302 BORDER 5:PAPER 5:INK 8:CLS
304 PRINT AT(2,4);"TESTMODUS - TEIL 3"
306 PRINT AT(5,4);"ZUFALLSZAHL 0=ZUFALL:"
308 INPUT SE
310 IF SE<0 THEN SE=-SE
312 IF SE=0 THEN SE=INT(RND(1)*9000)+1000
314 H9$="GEWAEHLT: "+MID$(STR$(SE),2):PRINT AT(8,4);H9$
316 PRINT AT(9,4);"WIEVIEL FRAGEN 1-98:"
318 INPUT TC
320 IF TC<1 THEN TC=1
322 IF TC>98 THEN TC=98
324 RR=SE-INT(SE/65536)*65536
326 FOR I=1 TO 98:P(I)=I:NEXT
328 FOR I=98 TO 2 STEP -1:GOSUB 1500:J=INT(RV*I)+1:T=P(I):P(I)=P(J):P(J)=T:NEXT
330 FOR QI=1 TO TC
332 KK=P(QI):GOSUB 1000
334 FOR I=1 TO 4:V(I)=I:NEXT
336 FOR I=4 TO 2 STEP -1:GOSUB 1500:J=INT(RV*I)+1:T=V(I):V(I)=V(J):V(J)=T:NEXT
338 CP=0:FOR M=1 TO 4:IF V(M)=C THEN CP=M
340 NEXT M
342 HE$="TEIL 3  FRAGE "+MID$(STR$(QI),2)+"/"+MID$(STR$(TC),2)+"  RICHTIG ":HD$=HE$+MID$(STR$(S),2)
344 GOSUB 1100
346 GOSUB 1400
348 GOSUB 1600
350 IF AC=2 THEN 100
352 NEXT QI
354 BORDER 5:PAPER 5:INK 8:CLS
356 PRINT AT(4,6);"ERGEBNIS - TEIL 3"
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
2000 DATA "Welche der folgenden Auflistungen enthaelt nur Bundeslaender, die zum Gebiet der frueheren DDR gehoerten?",2
2001 DATA "Niedersachsen, Nordrhein-Westfalen, Hessen, Schleswig-Holstein, Brandenburg"
2002 DATA "Mecklenburg-Vorpommern, Brandenburg, Sachsen, Sachsen-Anhalt, Thueringen"
2003 DATA "Bayern, Baden-Wuerttemberg, Rheinland-Pfalz, Thueringen, Sachsen"
2004 DATA "Sachsen, Thueringen, Hessen, Niedersachsen, Brandenburg"
2005 DATA "Zu wem gehoerte die DDR im 'Kalten Krieg'?",2
2006 DATA "zu den Westmaechten"
2007 DATA "zum Warschauer Pakt"
2008 DATA "zur NATO"
2009 DATA "zu den blockfreien Staaten"
2010 DATA "Wie hiess das Wirtschaftssystem der DDR?",2
2011 DATA "Marktwirtschaft"
2012 DATA "Planwirtschaft"
2013 DATA "Angebot und Nachfrage"
2014 DATA "Kapitalismus"
2015 DATA "Wie wurden die Bundesrepublik Deutschland und die DDR zu einem Staat?",2
2016 DATA "Die Bundesrepublik Deutschland hat die DDR besetzt."
2017 DATA "Die heutigen fuenf oestlichen Bundeslaender sind der Bundesrepublik Deutschland beigetreten."
2018 DATA "Die westlichen Bundeslaender sind der DDR beigetreten."
2019 DATA "Die DDR hat die Bundesrepublik Deutschland besetzt."
2020 DATA "Mit dem Beitritt der DDR zur Bundesrepublik Deutschland gehoeren die neuen Bundeslaender nun auch ...",1
2021 DATA "zur Europaeischen Union."
2022 DATA "zum Warschauer Pakt."
2023 DATA "zur OPEC."
2024 DATA "zur Europaeischen Verteidigungsgemeinschaft."
2025 DATA "Woran erinnern die sogenannten 'Stolpersteine' in Deutschland?",2
2026 DATA "an beruehmte deutsche Politikerinnen und Politiker"
2027 DATA "an die Opfer des Nationalsozialismus"
2028 DATA "an Verkehrstote"
2029 DATA "an bekannte juedische Musiker"
2030 DATA "In welchem Militaerbuendnis war die DDR Mitglied?",3
2031 DATA "in der NATO"
2032 DATA "im Rheinbund"
2033 DATA "im Warschauer Pakt"
2034 DATA "im Europabuendnis"
2035 DATA "Was war die 'Stasi'?",3
2036 DATA "der Geheimdienst im 'Dritten Reich'"
2037 DATA "eine beruehmte deutsche Gedenkstaette"
2038 DATA "der Geheimdienst der DDR"
2039 DATA "ein deutscher Sportverein waehrend des Zweiten Weltkrieges"
2040 DATA "Was ereignete sich am 17. Juni 1953 in der DDR?",2
2041 DATA "der feierliche Beitritt zum Warschauer Pakt"
2042 DATA "landesweite Streiks und ein Volksaufstand"
2043 DATA "der 1. SED-Parteitag"
2044 DATA "der erste Besuch Fidel Castros"
2045 DATA "Welcher Politiker steht fuer die 'Ostvertraege'?",2
2046 DATA "Helmut Kohl"
2047 DATA "Willy Brandt"
2048 DATA "Michail Gorbatschow"
2049 DATA "Ludwig Erhard"
2050 DATA "Wie heisst Deutschland mit vollem Namen?",3
2051 DATA "Bundesstaat Deutschland"
2052 DATA "Bundeslaender Deutschland"
2053 DATA "Bundesrepublik Deutschland"
2054 DATA "Bundesbezirk Deutschland"
2055 DATA "Wie viele Einwohner hat Deutschland?",3
2056 DATA "70 Millionen"
2057 DATA "78 Millionen"
2058 DATA "84 Millionen"
2059 DATA "90 Millionen"
2060 DATA "Welche Farben hat die deutsche Flagge?",1
2061 DATA "schwarz-rot-gold"
2062 DATA "rot-weiss-schwarz"
2063 DATA "schwarz-rot-gruen"
2064 DATA "schwarz-gelb-rot"
2065 DATA "Wer wird als 'Kanzler der Deutschen Einheit' bezeichnet?",2
2066 DATA "Gerhard Schroeder"
2067 DATA "Helmut Kohl"
2068 DATA "Konrad Adenauer"
2069 DATA "Helmut Schmidt"
2070 DATA "Welches Symbol ist im Plenarsaal des Deutschen Bundestages zu sehen?",2
2071 DATA "die Fahne der Stadt Berlin."
2072 DATA "der Bundesadler."
2073 DATA "der Reichsadler."
2074 DATA "die Reichskrone."
2075 DATA "In welchem Zeitraum gab es die Deutsche Demokratische Republik (DDR)?",3
2076 DATA "1919 bis 1927"
2077 DATA "1933 bis 1945"
2078 DATA "1949 bis 1990"
2079 DATA "1945 bis 1961"
2080 DATA "Wie viele Bundeslaender kamen bei der Wiedervereinigung 1990 zur Bundesrepublik Deutschland hinzu?",2
2081 DATA "4"
2082 DATA "5"
2083 DATA "6"
2084 DATA "7"
2085 DATA "Die Bundesrepublik Deutschland hat die Grenzen von heute seit ...",4
2086 DATA "1933."
2087 DATA "1949."
2088 DATA "1971."
2089 DATA "1990."
2090 DATA "Der 27. Januar ist in Deutschland ein offizieller Gedenktag. Woran erinnert dieser Tag?",4
2091 DATA "an das Ende des Zweiten Weltkrieges"
2092 DATA "an die Verabschiedung des Grundgesetzes"
2093 DATA "an die Wiedervereinigung Deutschlands"
2094 DATA "an die Opfer des Nationalsozialismus (Tag der Befreiung des Vernichtungslagers Auschwitz)"
2095 DATA "Deutschland ist Mitglied des Schengener Abkommens. Was bedeutet das?",1
2096 DATA "Deutsche koennen in viele Laender Europas ohne Passkontrolle reisen."
2097 DATA "Alle Menschen koennen ohne Personenkontrolle in Deutschland einreisen."
2098 DATA "Deutsche koennen ohne Passkontrolle in jedes Land reisen."
2099 DATA "Deutsche koennen in jedem Land mit dem Euro bezahlen."
2100 DATA "Welches Land ist ein Nachbarland von Deutschland?",4
2101 DATA "Ungarn"
2102 DATA "Portugal"
2103 DATA "Spanien"
2104 DATA "Schweiz"
2105 DATA "Welches Land ist ein Nachbarland von Deutschland?",3
2106 DATA "Rumaenien"
2107 DATA "Bulgarien"
2108 DATA "Polen"
2109 DATA "Griechenland"
2110 DATA "Was bedeutet die Abkuerzung EU?",2
2111 DATA "Europaeische Unternehmen"
2112 DATA "Europaeische Union"
2113 DATA "Einheitliche Union"
2114 DATA "Euro Union"
2115 DATA "In welchem anderen Land gibt es eine grosse deutschsprachige Bevoelkerung?",4
2116 DATA "Tschechien"
2117 DATA "Norwegen"
2118 DATA "Spanien"
2119 DATA "Oesterreich"
2120 DATA "Welches Land ist ein Nachbarland von Deutschland?",2
2121 DATA "Finnland"
2122 DATA "Daenemark"
2123 DATA "Norwegen"
2124 DATA "Schweden"
2125 DATA "Wie wird der Beitritt der DDR zur Bundesrepublik Deutschland im Jahr 1990 allgemein genannt?",4
2126 DATA "NATO-Osterweiterung"
2127 DATA "EU-Osterweiterung"
2128 DATA "Europaeische Gemeinschaft"
2129 DATA "Deutsche Wiedervereinigung"
2130 DATA "Welches Land ist ein Nachbarland von Deutschland?",4
2131 DATA "Spanien"
2132 DATA "Bulgarien"
2133 DATA "Norwegen"
2134 DATA "Luxemburg"
2135 DATA "Das Europaeische Parlament wird regelmaessig gewaehlt, naemlich alle ...",1
2136 DATA "5 Jahre."
2137 DATA "6 Jahre."
2138 DATA "7 Jahre."
2139 DATA "8 Jahre."
2140 DATA "Was bedeutet der Begriff 'europaeische Integration'?",4
2141 DATA "Damit sind amerikanische Einwanderer in Europa gemeint."
2142 DATA "Der Begriff meint den Einwanderungsstopp nach Europa."
2143 DATA "Damit sind europaeische Auswanderer in den USA gemeint."
2144 DATA "Der Begriff meint den Zusammenschluss europaeischer Staaten zur EU."
2145 DATA "Wer wird bei der Europawahl gewaehlt?",4
2146 DATA "die Europaeische Kommission"
2147 DATA "die Laender, die in die EU eintreten duerfen"
2148 DATA "die europaeische Verfassung"
2149 DATA "die Abgeordneten des Europaeischen Parlaments"
2150 DATA "Welches Land ist ein Nachbarland von Deutschland?",1
2151 DATA "Tschechien"
2152 DATA "Bulgarien"
2153 DATA "Griechenland"
2154 DATA "Portugal"
2155 DATA "Wo ist ein Sitz des Europaeischen Parlaments?",4
2156 DATA "London"
2157 DATA "Paris"
2158 DATA "Berlin"
2159 DATA "Strassburg"
2160 DATA "Der franzoesische Staatspraesident Francois Mitterrand und der deutsche Bundeskanzler Helmut Kohl gedenken in Verdun gemeinsam der Toten beider Weltkriege. Welches Ziel der Europaeischen Union wird bei diesem Treffen deutlich?",3
2161 DATA "Freundschaft zwischen England und Deutschland"
2162 DATA "Reisefreiheit in alle Laender der EU"
2163 DATA "Frieden und Sicherheit in den Laendern der EU"
2164 DATA "einheitliche Feiertage in den Laendern der EU"
2165 DATA "Wie viele Mitgliedstaaten hat die EU heute?",4
2166 DATA "21"
2167 DATA "23"
2168 DATA "25"
2169 DATA "27"
2170 DATA "2007 wurde das 50-jaehrige Jubilaeum der 'Roemischen Vertraege' gefeiert. Was war der Inhalt der Vertraege?",2
2171 DATA "Beitritt Deutschlands zur NATO"
2172 DATA "Gruendung der Europaeischen Wirtschaftsgemeinschaft (EWG)"
2173 DATA "Verpflichtung Deutschlands zu Reparationsleistungen"
2174 DATA "Festlegung der Oder-Neisse-Linie als Ostgrenze"
2175 DATA "An welchen Orten arbeitet das Europaeische Parlament?",2
2176 DATA "Paris, London und Den Haag"
2177 DATA "Strassburg, Luxemburg und Bruessel"
2178 DATA "Rom, Bern und Wien"
2179 DATA "Bonn, Zuerich und Mailand"
2180 DATA "Durch welche Vertraege schloss sich die Bundesrepublik Deutschland mit anderen Staaten zur Europaeischen Wirtschaftsgemeinschaft zusammen?",2
2181 DATA "durch die 'Hamburger Vertraege'"
2182 DATA "durch die 'Roemischen Vertraege'"
2183 DATA "durch die 'Pariser Vertraege'"
2184 DATA "durch die 'Londoner Vertraege'"
2185 DATA "Seit wann bezahlt man in Deutschland mit dem Euro in bar?",3
2186 DATA "1995"
2187 DATA "1998"
2188 DATA "2002"
2189 DATA "2005"
2190 DATA "Frau Seger bekommt ein Kind. Was muss sie tun, um Elterngeld zu erhalten?",3
2191 DATA "Sie muss an ihre Krankenkasse schreiben."
2192 DATA "Sie muss nichts tun, denn sie bekommt automatisch Elterngeld."
2193 DATA "Sie muss einen Antrag bei der Elterngeldstelle stellen."
2194 DATA "Sie muss das Arbeitsamt um Erlaubnis bitten."
2195 DATA "Wer entscheidet, ob ein Kind in Deutschland in den Kindergarten geht?",3
2196 DATA "der Staat"
2197 DATA "die Bundeslaender"
2198 DATA "die Eltern / die Erziehungsberechtigten"
2199 DATA "die Schulen"
2200 DATA "Maik und Sybille wollen mit Freunden an ihrem deutschen Wohnort eine Demonstration auf der Strasse abhalten. Was muessen sie vorher tun?",2
2201 DATA "Sie muessen nichts tun. Man darf in Deutschland jederzeit ueberall demonstrieren."
2202 DATA "Sie muessen die Demonstration anmelden."
2203 DATA "Sie koennen gar nichts tun, denn Demonstrationen sind in Deutschland grundsaetzlich verboten."
2204 DATA "Maik und Sybille muessen einen neuen Verein gruenden, weil nur Vereine demonstrieren duerfen."
2205 DATA "Welchen Schulabschluss braucht man normalerweise, um an einer Universitaet in Deutschland ein Studium zu beginnen?",1
2206 DATA "das Abitur"
2207 DATA "ein Diplom"
2208 DATA "die Prokura"
2209 DATA "eine Gesellenpruefung"
2210 DATA "Wer darf in Deutschland nicht als Paar zusammenleben?",4
2211 DATA "Hans (20 Jahre) und Marie (19 Jahre)"
2212 DATA "Tom (20 Jahre) und Klaus (45 Jahre)"
2213 DATA "Sofie (35 Jahre) und Lisa (40 Jahre)"
2214 DATA "Anne (13 Jahre) und Tim (25 Jahre)"
2215 DATA "Ab welchem Alter ist man in Deutschland volljaehrig?",2
2216 DATA "16"
2217 DATA "18"
2218 DATA "19"
2219 DATA "21"
2220 DATA "Eine Frau ist schwanger. Sie ist kurz vor und nach der Geburt ihres Kindes vom Gesetz besonders beschuetzt. Wie heisst dieser Schutz?",3
2221 DATA "Elternzeit"
2222 DATA "Geburtsvorbereitung"
2223 DATA "Mutterschutz"
2224 DATA "Wochenbett"
2225 DATA "Die Erziehung der Kinder ist in Deutschland vor allem Aufgabe ...",2
2226 DATA "des Staates."
2227 DATA "der Eltern."
2228 DATA "der Grosseltern."
2229 DATA "der Schulen."
2230 DATA "Wer ist in Deutschland hauptsaechlich verantwortlich fuer die Kindererziehung?",2
2231 DATA "der Staat"
2232 DATA "die Eltern"
2233 DATA "die Verwandten"
2234 DATA "die Schulen"
2235 DATA "In Deutschland hat man die besten Chancen auf einen gut bezahlten Arbeitsplatz, wenn man ...",2
2236 DATA "katholisch ist."
2237 DATA "gut ausgebildet ist."
2238 DATA "eine Frau ist."
2239 DATA "Mitglied einer Partei ist."
2240 DATA "Wenn man in Deutschland ein Kind schlaegt, ...",4
2241 DATA "geht das niemanden etwas an."
2242 DATA "geht das nur die Familie etwas an."
2243 DATA "kann man dafuer nicht bestraft werden."
2244 DATA "kann man dafuer bestraft werden."
2245 DATA "In Deutschland ...",1
2246 DATA "darf man zur gleichen Zeit nur mit einem Partner / einer Partnerin verheiratet sein."
2247 DATA "kann man mehrere Ehepartner / Ehepartnerinnen gleichzeitig haben."
2248 DATA "darf man nicht wieder heiraten, wenn man einmal verheiratet war."
2249 DATA "darf eine Frau nicht wieder heiraten, wenn ihr Mann gestorben ist."
2250 DATA "Wo muessen Sie sich anmelden, wenn Sie in Deutschland umziehen?",1
2251 DATA "beim Einwohnermeldeamt"
2252 DATA "beim Standesamt"
2253 DATA "beim Ordnungsamt"
2254 DATA "beim Gewerbeamt"
2255 DATA "In Deutschland duerfen Ehepaare sich scheiden lassen. Meistens muessen sie dazu das 'Trennungsjahr' einhalten. Was bedeutet das?",4
2256 DATA "Der Scheidungsprozess dauert ein Jahr."
2257 DATA "Die Ehegatten sind ein Jahr verheiratet, dann ist die Scheidung moeglich."
2258 DATA "Das Besuchsrecht fuer die Kinder gilt ein Jahr."
2259 DATA "Die Ehegatten fuehren mindestens ein Jahr getrennt ihr eigenes Leben. Danach ist die Scheidung moeglich."
2260 DATA "Bei Erziehungsproblemen koennen Eltern in Deutschland Hilfe erhalten vom ...",3
2261 DATA "Ordnungsamt."
2262 DATA "Schulamt."
2263 DATA "Jugendamt."
2264 DATA "Gesundheitsamt."
2265 DATA "Ein Ehepaar moechte in Deutschland ein Restaurant eroeffnen. Was braucht es dazu unbedingt?",4
2266 DATA "eine Erlaubnis der Polizei"
2267 DATA "eine Genehmigung einer Partei"
2268 DATA "eine Genehmigung des Einwohnermeldeamts"
2269 DATA "eine Gaststaettenerlaubnis von der zustaendigen Behoerde"
2270 DATA "Eine erwachsene Frau moechte in Deutschland das Abitur nachholen. Das kann sie an ...",2
2271 DATA "einer Hochschule."
2272 DATA "einem Abendgymnasium."
2273 DATA "einer Hauptschule."
2274 DATA "einer Privatuniversitaet."
2275 DATA "Was darf das Jugendamt in Deutschland?",2
2276 DATA "Es entscheidet, welche Schule das Kind besucht."
2277 DATA "Es kann ein Kind, das geschlagen wird oder hungern muss, aus der Familie nehmen."
2278 DATA "Es bezahlt das Kindergeld an die Eltern."
2279 DATA "Es kontrolliert, ob das Kind einen Kindergarten besucht."
2280 DATA "Das Berufsinformationszentrum BIZ bei der Bundesagentur fuer Arbeit in Deutschland hilft bei der ...",2
2281 DATA "Rentenberechnung."
2282 DATA "Lehrstellensuche."
2283 DATA "Steuererklaerung."
2284 DATA "Krankenversicherung."
2285 DATA "In Deutschland hat ein Kind in der Schule ...",4
2286 DATA "Recht auf unbegrenzte Freizeit."
2287 DATA "Wahlfreiheit fuer alle Faecher."
2288 DATA "Anspruch auf Schulgeld."
2289 DATA "Anwesenheitspflicht."
2290 DATA "Ein Mann moechte mit 30 Jahren in Deutschland sein Abitur nachholen. Wo kann er das tun? An ...",2
2291 DATA "einer Hochschule"
2292 DATA "einem Abendgymnasium"
2293 DATA "einer Hauptschule"
2294 DATA "einer Privatuniversitaet"
2295 DATA "Was bedeutet in Deutschland der Grundsatz der Gleichbehandlung?",1
2296 DATA "Niemand darf z. B. wegen einer Behinderung benachteiligt werden."
2297 DATA "Man darf andere Personen benachteiligen, wenn ausreichende persoenliche Gruende hierfuer vorliegen."
2298 DATA "Niemand darf gegen Personen klagen, wenn sie benachteiligt wurden."
2299 DATA "Es ist fuer alle Gesetz, benachteiligten Gruppen jaehrlich Geld zu spenden."
2300 DATA "In Deutschland sind Jugendliche ab 14 Jahren strafmuendig. Das bedeutet: Jugendliche, die 14 Jahre und aelter sind und gegen Strafgesetze verstossen, ...",1
2301 DATA "werden bestraft."
2302 DATA "werden wie Erwachsene behandelt."
2303 DATA "teilen die Strafe mit ihren Eltern."
2304 DATA "werden nicht bestraft."
2305 DATA "Zu welchem Fest tragen Menschen in Deutschland bunte Kostueme und Masken?",1
2306 DATA "am Rosenmontag"
2307 DATA "am Maifeiertag"
2308 DATA "beim Oktoberfest"
2309 DATA "an Pfingsten"
2310 DATA "Wohin muss man in Deutschland zuerst gehen, wenn man heiraten moechte?",4
2311 DATA "zum Einwohnermeldeamt"
2312 DATA "zum Ordnungsamt"
2313 DATA "zur Agentur fuer Arbeit"
2314 DATA "zum Standesamt"
2315 DATA "Wann beginnt die gesetzliche Nachtruhe in Deutschland?",2
2316 DATA "wenn die Sonne untergeht"
2317 DATA "um 22 Uhr"
2318 DATA "wenn die Nachbarn schlafen gehen"
2319 DATA "um 0 Uhr, Mitternacht"
2320 DATA "Eine junge Frau in Deutschland, 22 Jahre alt, lebt mit ihrem Freund zusammen. Die Eltern der Frau finden das nicht gut, weil ihnen der Freund nicht gefaellt. Was koennen die Eltern tun?",1
2321 DATA "Sie muessen die Entscheidung der volljaehrigen Tochter respektieren."
2322 DATA "Sie haben das Recht, die Tochter in die elterliche Wohnung zurueckzuholen."
2323 DATA "Sie koennen zur Polizei gehen und die Tochter anzeigen."
2324 DATA "Sie suchen einen anderen Mann fuer die Tochter."
2325 DATA "Eine junge Frau will den Fuehrerschein machen. Sie hat Angst vor der Pruefung, weil ihre Muttersprache nicht Deutsch ist. Was ist richtig?",2
2326 DATA "Sie muss mindestens zehn Jahre in Deutschland leben, bevor sie den Fuehrerschein machen kann."
2327 DATA "Sie kann die Theorie-Pruefung vielleicht in ihrer Muttersprache machen. Es gibt mehr als zehn Sprachen zur Auswahl."
2328 DATA "Wenn sie kein Deutsch kann, darf sie keinen Fuehrerschein haben."
2329 DATA "Sie muss den Fuehrerschein in dem Land machen, in dem man ihre Sprache spricht."
2330 DATA "In Deutschland haben Kinder ab dem Alter von drei Jahren bis zur Ersteinschulung einen Anspruch auf ...",2
2331 DATA "monatliches Taschengeld."
2332 DATA "einen Kindergartenplatz."
2333 DATA "einen Platz in einem Sportverein."
2334 DATA "einen Ferienpass."
2335 DATA "Die Volkshochschule in Deutschland ist eine Einrichtung ...",3
2336 DATA "fuer den Religionsunterricht."
2337 DATA "nur fuer Jugendliche."
2338 DATA "zur Weiterbildung."
2339 DATA "nur fuer Rentner und Rentnerinnen."
2340 DATA "Was ist in Deutschland ein Brauch zu Weihnachten?",2
2341 DATA "bunte Eier verstecken"
2342 DATA "einen Tannenbaum schmuecken"
2343 DATA "sich mit Masken und Kostuemen verkleiden"
2344 DATA "Kuerbisse vor die Tuer stellen"
2345 DATA "Welche Lebensform ist in Deutschland nicht erlaubt?",4
2346 DATA "Mann und Frau sind geschieden und leben mit neuen Partnern zusammen."
2347 DATA "Zwei Frauen leben zusammen."
2348 DATA "Ein allein erziehender Vater lebt mit seinen zwei Kindern zusammen."
2349 DATA "Ein Mann ist mit zwei Frauen zur selben Zeit verheiratet."
2350 DATA "Bei Erziehungsproblemen gehen Sie in Deutschland ...",4
2351 DATA "zum Arzt / zur Aerztin."
2352 DATA "zum Gesundheitsamt."
2353 DATA "zum Einwohnermeldeamt."
2354 DATA "zum Jugendamt."
2355 DATA "Sie haben in Deutschland absichtlich einen Brief geoeffnet, der an eine andere Person adressiert ist. Was haben Sie nicht beachtet?",2
2356 DATA "das Schweigerecht"
2357 DATA "das Briefgeheimnis"
2358 DATA "die Schweigepflicht"
2359 DATA "die Meinungsfreiheit"
2360 DATA "Was braucht man in Deutschland fuer eine Ehescheidung?",4
2361 DATA "die Einwilligung der Eltern"
2362 DATA "ein Attest eines Arztes / einer Aerztin"
2363 DATA "die Einwilligung der Kinder"
2364 DATA "die Unterstuetzung eines Anwalts / einer Anwaeltin"
2365 DATA "Was sollten Sie tun, wenn Sie von Ihrem Ansprechpartner / Ihrer Ansprechpartnerin in einer deutschen Behoerde schlecht behandelt werden?",4
2366 DATA "Ich kann nichts tun."
2367 DATA "Ich muss mir diese Behandlung gefallen lassen."
2368 DATA "Ich drohe der Person."
2369 DATA "Ich kann mich beim Behoerdenleiter / bei der Behoerdenleiterin beschweren."
2370 DATA "Eine Frau, die ein zweijaehriges Kind hat, bewirbt sich in Deutschland um eine Stelle. Was ist ein Beispiel fuer Diskriminierung? Sie bekommt die Stelle nur deshalb nicht, weil sie ...",4
2371 DATA "kein Englisch spricht."
2372 DATA "zu hohe Gehaltsvorstellungen hat."
2373 DATA "keine Erfahrungen in diesem Beruf hat."
2374 DATA "Mutter ist."
2375 DATA "Ein Mann im Rollstuhl hat sich auf eine Stelle als Buchhalter beworben. Was ist ein Beispiel fuer Diskriminierung? Er bekommt die Stelle nur deshalb nicht, weil er ...",1
2376 DATA "im Rollstuhl sitzt."
2377 DATA "keine Erfahrung hat."
2378 DATA "zu hohe Gehaltsvorstellungen hat."
2379 DATA "kein Englisch spricht."
2380 DATA "In den meisten Mietshaeusern in Deutschland gibt es eine 'Hausordnung'. Was steht in einer solchen 'Hausordnung'? Sie nennt ...",3
2381 DATA "Regeln fuer die Benutzung oeffentlicher Verkehrsmittel."
2382 DATA "alle Mieter und Mieterinnen im Haus."
2383 DATA "Regeln, an die sich alle Bewohner und Bewohnerinnen halten muessen."
2384 DATA "die Adresse des naechsten Ordnungsamtes."
2385 DATA "Wenn Sie sich in Deutschland gegen einen falschen Steuerbescheid wehren wollen, muessen Sie ...",3
2386 DATA "nichts machen."
2387 DATA "den Bescheid wegwerfen."
2388 DATA "Einspruch einlegen."
2389 DATA "warten, bis ein anderer Bescheid kommt."
2390 DATA "Zwei Freunde wollen in ein oeffentliches Schwimmbad in Deutschland. Beide haben eine dunkle Hautfarbe und werden deshalb nicht hineingelassen. Welches Recht wird in dieser Situation verletzt? Das Recht auf ...",2
2391 DATA "Meinungsfreiheit"
2392 DATA "Gleichbehandlung"
2393 DATA "Versammlungsfreiheit"
2394 DATA "Freizuegigkeit"
2395 DATA "Welches Ehrenamt muessen deutsche Staatsbuerger / Staatsbuergerinnen uebernehmen, wenn sie dazu aufgefordert werden?",3
2396 DATA "Vereinstrainer / Vereinstrainerin"
2397 DATA "Bibliotheksaufsicht"
2398 DATA "Wahlhelfer / Wahlhelferin"
2399 DATA "Lehrer / Lehrerin"
2400 DATA "Was tun Sie, wenn Sie eine falsche Rechnung von einer deutschen Behoerde bekommen?",2
2401 DATA "Ich lasse die Rechnung liegen."
2402 DATA "Ich lege Widerspruch bei der Behoerde ein."
2403 DATA "Ich schicke die Rechnung an die Behoerde zurueck."
2404 DATA "Ich gehe mit der Rechnung zum Finanzamt."
2405 DATA "Was man fuer die Arbeit koennen muss, aendert sich in Zukunft sehr schnell. Was kann man tun?",3
2406 DATA "Es ist egal, was man lernt."
2407 DATA "Kinder lernen in der Schule alles, was im Beruf wichtig ist. Nach der Schule muss man nicht weiter lernen."
2408 DATA "Erwachsene muessen auch nach der Ausbildung immer weiter lernen."
2409 DATA "Alle muessen frueher aufhoeren zu arbeiten, weil sich alles aendert."
2410 DATA "Frau Frost arbeitet als fest angestellte Mitarbeiterin in einem Buero. Was muss sie nicht von ihrem Gehalt bezahlen?",1
2411 DATA "Umsatzsteuer"
2412 DATA "Lohnsteuer"
2413 DATA "Beitraege zur Arbeitslosenversicherung"
2414 DATA "Beitraege zur Renten- und Krankenversicherung"
2415 DATA "Welche Organisation in einer Firma hilft den Arbeitnehmern und Arbeitnehmerinnen bei Problemen mit dem Arbeitgeber / der Arbeitgeberin?",1
2416 DATA "der Betriebsrat"
2417 DATA "der Betriebspruefer / die Betriebsprueferin"
2418 DATA "die Betriebsgruppe"
2419 DATA "das Betriebsmanagement"
2420 DATA "Sie moechten bei einer Firma in Deutschland Ihr Arbeitsverhaeltnis beenden. Was muessen Sie beachten?",3
2421 DATA "die Gehaltszahlungen"
2422 DATA "die Arbeitszeit"
2423 DATA "die Kuendigungsfrist"
2424 DATA "die Versicherungspflicht"
2425 DATA "Woraus begruendet sich Deutschlands besondere Verantwortung fuer Israel?",2
2426 DATA "aus der Mitgliedschaft in der Europaeischen Union (EU)"
2427 DATA "aus den nationalsozialistischen Verbrechen gegen Juden"
2428 DATA "aus dem Grundgesetz der Bundesrepublik Deutschland"
2429 DATA "aus der christlichen Tradition"
2430 DATA "Ein Mann mit dunkler Hautfarbe bewirbt sich um eine Stelle als Kellner in einem Restaurant in Deutschland. Was ist ein Beispiel fuer Diskriminierung? Er bekommt die Stelle nur deshalb nicht, weil ...",3
2431 DATA "seine Deutschkenntnisse zu gering sind."
2432 DATA "er zu hohe Gehaltsvorstellungen hat."
2433 DATA "er eine dunkle Haut hat."
2434 DATA "er keine Erfahrungen im Beruf hat."
2435 DATA "Sie haben in Deutschland einen Fernseher gekauft. Zu Hause packen Sie den Fernseher aus, doch er funktioniert nicht. Der Fernseher ist kaputt. Was koennen Sie machen?",2
2436 DATA "eine Anzeige schreiben"
2437 DATA "den Fernseher reklamieren"
2438 DATA "das Geraet ungefragt austauschen"
2439 DATA "die Garantie verlaengern"
2440 DATA "Warum muss man in Deutschland bei der Steuererklaerung aufschreiben, ob man zu einer Kirche gehoert oder nicht? Weil ...",2
2441 DATA "das fuer die Statistik in Deutschland wichtig ist."
2442 DATA "es eine Kirchensteuer gibt, die an die Einkommen- und Lohnsteuer geknuepft ist."
2443 DATA "man mehr Steuern zahlen muss, wenn man nicht zu einer Kirche gehoert."
2444 DATA "die Kirche fuer die Steuererklaerung verantwortlich ist."
2445 DATA "Die Menschen in Deutschland leben nach dem Grundsatz der religioesen Toleranz. Was bedeutet das?",3
2446 DATA "Es duerfen keine Moscheen gebaut werden."
2447 DATA "Alle Menschen glauben an Gott."
2448 DATA "Jeder kann glauben, was er moechte."
2449 DATA "Der Staat entscheidet, an welchen Gott die Menschen glauben."
2450 DATA "Was ist in Deutschland ein Brauch an Ostern?",3
2451 DATA "Kuerbisse vor die Tuer stellen"
2452 DATA "einen Tannenbaum schmuecken"
2453 DATA "Eier bemalen"
2454 DATA "Raketen in die Luft schiessen"
2455 DATA "Pfingsten ist ein ...",1
2456 DATA "christlicher Feiertag."
2457 DATA "deutscher Gedenktag."
2458 DATA "internationaler Trauertag."
2459 DATA "bayerischer Brauch."
2460 DATA "Welche Religion hat die europaeische und deutsche Kultur gepraegt?",2
2461 DATA "der Hinduismus"
2462 DATA "das Christentum"
2463 DATA "der Buddhismus"
2464 DATA "der Islam"
2465 DATA "In Deutschland nennt man die letzten vier Wochen vor Weihnachten ...",3
2466 DATA "den Buss- und Bettag."
2467 DATA "das Erntedankfest."
2468 DATA "die Adventszeit."
2469 DATA "Allerheiligen."
2470 DATA "Aus welchem Land sind die meisten Migranten / Migrantinnen nach Deutschland gekommen?",4
2471 DATA "Italien"
2472 DATA "Polen"
2473 DATA "Marokko"
2474 DATA "Tuerkei"
2475 DATA "In der DDR lebten vor allem Migranten aus ...",1
2476 DATA "Vietnam, Polen, Mosambik."
2477 DATA "Frankreich, Rumaenien, Somalia."
2478 DATA "Chile, Ungarn, Simbabwe."
2479 DATA "Nordkorea, Mexiko, Aegypten."
2480 DATA "Auslaendische Arbeitnehmer und Arbeitnehmerinnen, die in den 50er und 60er Jahren von der Bundesrepublik Deutschland angeworben wurden, nannte man ...",2
2481 DATA "Schwarzarbeiter / Schwarzarbeiterinnen"
2482 DATA "Gastarbeiter / Gastarbeiterinnen"
2483 DATA "Zeitarbeiter / Zeitarbeiterinnen"
2484 DATA "Schichtarbeiter / Schichtarbeiterinnen"
2485 DATA "Aus welchem Land kamen die ersten Gastarbeiter / Gastarbeiterinnen nach Deutschland?",1
2486 DATA "Italien"
2487 DATA "Spanien"
2488 DATA "Portugal"
2489 DATA "Tuerkei"
