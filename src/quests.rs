//! The text of the quest journal (journal.rs): the quests of the game and
//! their entries, each with the condition that makes it appear.
//!
//! A condition is made of terms joined by `&` (all of them) and `|` (one of
//! the groups). A term is the name of a global flag (set, i.e. not 0),
//! `name=n` (the flag has exactly that value), `has:shape:frame` (the
//! Avatar carries such a thing) or `dead:npc` (that person is dead); `!` in
//! front turns it around. An entry
//! stays in the journal once reached, unless its condition has a `!`: then
//! it disappears again when the condition no longer holds.
//!
//! The texts only note what the Avatar has seen or was told in the game.
//! German follows the old spelling of the game (daß, muß).

pub struct Stage {
    pub when: &'static str,
    pub de: &'static str,
    pub en: &'static str,
}

pub struct Quest {
    pub de: &'static str,
    pub en: &'static str,
    /// Condition of the entry that finishes the quest ("" = never).
    pub done: &'static str,
    pub stages: &'static [Stage],
}

/// The fragments of the great obelisk (shape 442) and, after they were
/// used against their Titans, the glowing fragments (shape 793); frames:
/// 0 tip, 1 Heart of the Earth, 2 Tear of the Seas, 3 Breath of the Wind,
/// 4 Tongue of Flame.
pub const QUESTS: &[Quest] = &[
    Quest {
        de: "Ankunft in Pagan",
        en: "Arrival in Pagan",
        done: "toranDead",
        stages: &[
            Stage {
                when: "devonMet",
                de: "Ich bin in einer fremden Welt erwacht, im Meer treibend. Ein Fischer namens Devon hat mich an Land gezogen. Diese Welt heißt Pagan, die Stadt in der Nähe Tenebrae. Devon warnte mich: Tenebrae ist ein Ort der Gewalt.",
                en: "I woke in a strange world, adrift in the sea. A fisherman named Devon pulled me ashore. This world is called Pagan, the town nearby Tenebrae. Devon warned me: Tenebrae is a place of violence.",
            },
            Stage {
                when: "toranDead",
                de: "Am Bootssteg wurde ein Mann namens Toran hingerichtet, weil er die Herrscherin Mordea verleumdet haben soll. Mordea verwehrte ihm sogar das Begräbnis und ließ ihn ins Meer werfen.",
                en: "On the dock a man named Toran was executed for slandering the ruler, Mordea. She even denied him a burial and had him thrown into the sea.",
            },
        ],
    },
    Quest {
        de: "Der Weg nach Hause",
        en: "The way home",
        done: "",
        stages: &[
            Stage {
                when: "devonMet",
                de: "Ich muß einen Weg finden, Pagan zu verlassen und nach Britannia zurückzukehren. Vielleicht weiß in Tenebrae jemand Rat.",
                en: "I must find a way to leave Pagan and return to Britannia. Perhaps someone in Tenebrae can help.",
            },
            Stage {
                when: "benticSaidMythran & !mythranMet",
                de: "Der Gelehrte Bentic erwähnte einen Magier namens Mythran, der außerhalb der Stadt lebt.",
                en: "The scholar Bentic mentioned a mage named Mythran who lives outside the town.",
            },
            Stage {
                when: "mythranMet",
                de: "Ich habe den Magier Mythran in seinem seltsamen Haus besucht. Er weiß viel über Pagan und seine Magie.",
                en: "I visited the mage Mythran in his strange house. He knows much about Pagan and its magic.",
            },
            Stage {
                when: "mythDoneLeave",
                de: "Mythran glaubt, daß die Macht der vier Titanen den Weg in andere Welten versperrt. Um Pagan verlassen zu können, muß ich wohl selbst die Macht aller vier Titanen erlangen. Er riet mir, mit den Nekromanten zu beginnen: Sie leben auf dem Friedhof, an der Straße, die aus der Stadt nach Osten führt.",
                en: "Mythran believes the power of the four Titans bars the way to other worlds. To leave Pagan I must probably gain the power of all four Titans myself. He advised me to begin with the necromancers: they live in the cemetery, along the road that leads east out of town.",
            },
            Stage {
                when: "isApprentice",
                de: "Ich bin Lehrling des Nekromanten Vividos geworden. Mein Weg zur Macht des Erdtitanen Lithos hat begonnen.",
                en: "I have become the apprentice of the necromancer Vividos. My path to the power of Lithos, the Titan of Earth, has begun.",
            },
            Stage {
                when: "blackLoreSeen",
                de: "Im Buch 'Die Zerstörung des Tempels' las ich vom Großen Tempel auf dem Berg Morgaelin und seinem Obelisken aus Schwarzgestein. Durch ihn erhielten die Elemente ihre Macht und wurden zu Titanen. Der Zerstörer zerschmetterte ihn in fünf Bruchstücke: Lithos soll das Herz der Erde in der Totengruft aufbewahren, Hydros legte die Träne der Meere auf den Meeresgrund, der Atem des Windes soll auf der Insel Silberfels sein, und das vierte Stück gaben die ersten Hexer nicht mehr her, um damit den Feuertitanen zu binden. Wo die Spitze liegt, weiß niemand. Wer alle Teile nahe dem Großen Tempel zusammenfügt und ihnen wieder Kraft zuführt, könnte die Macht des Obelisken erneuern.",
                en: "In the book 'The Destruction of the Temple' I read of the Great Temple on Mount Morgaelin and its obelisk of blackrock. Through it the elements gained their power and became Titans. The Destroyer shattered it into five fragments: Lithos is said to keep the Heart of Earth in the Tomb of the Dead, Hydros laid the Tear of the Seas on the bottom of the sea, the Breath of Wind is said to be on Argentrock Isle, and the fourth piece was kept by the first sorcerers to bind the Titan of Fire. Where the tip lies, nobody knows. Whoever joins all the pieces near the Great Temple and gives them power again might renew the power of the obelisk.",
            },
            Stage {
                when: "ancientOnesMet",
                de: "Ich bin den Alten begegnet. Sie kennen mich, wissen, woher ich komme, und wollen mir helfen. Hinter ihnen liegt die Grabstätte von Khumash-Gor, dem gewaltigsten Krieger aller Zeiten. Bei ihm liegt ein Gegenstand aus Schwarzgestein in Form einer kleinen Pyramide. Selbst im Tode wird Khumash-Gor ihn nicht freiwillig hergeben.",
                en: "I met the Ancient Ones. They know me and where I come from, and they want to help me. Behind them lies the tomb of Khumash-Gor, the mightiest warrior of all time. With him lies an object of blackrock shaped like a small pyramid. Even in death Khumash-Gor will not give it up willingly.",
            },
            Stage {
                when: "getPowerAO",
                de: "Ich habe die Spitze des Obelisken. Die Alten sagen, es gebe ein fünftes Element, den Äther, und es sei mir bestimmt, sein Titan zu werden. Dazu muß ich die anderen Titanen entmachten und die Quellen ihrer Macht finden. Hydros finde ich in ihrem unterirdischen Tempel, Stratos und Pyros leben an der Oberfläche.",
                en: "I have the tip of the obelisk. The Ancient Ones say there is a fifth element, the ether, and that I am destined to become its Titan. To do so I must strip the other Titans of their power and find the sources of it. Hydros dwells in her underground temple; Stratos and Pyros live on the surface.",
            },
            Stage {
                when: "has:442:0 & has:442:1 & has:442:2 & has:442:3 & has:442:4",
                de: "Ich habe alle fünf Bruchstücke des Obelisken beisammen.",
                en: "I have all five fragments of the obelisk.",
            },
            Stage {
                when: "has:793:1",
                de: "Mit dem Herz der Erde habe ich die Macht von Lithos gebrochen. Der Stein glüht nun.",
                en: "With the Heart of Earth I broke the power of Lithos. The stone now glows.",
            },
            Stage {
                when: "has:793:2",
                de: "Mit der Träne der Meere habe ich die Macht von Hydros gebrochen. Der Stein glüht nun.",
                en: "With the Tear of the Seas I broke the power of Hydros. The stone now glows.",
            },
            Stage {
                when: "has:793:3",
                de: "Mit dem Atem des Windes habe ich die Macht von Stratos gebrochen. Der Stein glüht nun.",
                en: "With the Breath of Wind I broke the power of Stratos. The stone now glows.",
            },
            Stage {
                when: "has:793:4",
                de: "Mit der Flammenzunge habe ich die Macht von Pyros gebrochen. Der Stein glüht nun.",
                en: "With the Tongue of Flame I broke the power of Pyros. The stone now glows.",
            },
            Stage {
                when: "has:793:0",
                de: "Nun glüht auch die Spitze des Obelisken. Ich kann meine Macht spüren.",
                en: "Now the tip of the obelisk glows as well. I can feel my power.",
            },
        ],
    },
    Quest {
        de: "Der Zeremoniendolch",
        en: "The ceremonial dagger",
        done: "lothianDead",
        stages: &[
            Stage {
                when: "vivTold",
                de: "Auf dem Friedhof traf ich Vividos, den Schüler der sterbenden Nekromantin Lothian. Mordea hat den Nekromanten ihren Zeremoniendolch genommen. Ohne ihn kann Vividos Lothian nicht zu Lithos schicken, und der Bergkönig wird zornig werden.",
                en: "In the cemetery I met Vividos, pupil of the dying necromancer Lothian. Mordea has taken the necromancers' ceremonial dagger. Without it Vividos cannot send Lothian to Lithos, and the Mountain King will grow angry.",
            },
            Stage {
                when: "toldDagger",
                de: "Ich habe Vividos versprochen, den Dolch zurückzuholen. Mordea bewahrt ihn in ihren Gemächern auf. Er hat einen goldenen Griff und eine schlangenförmige Klinge.",
                en: "I promised Vividos to bring back the dagger. Mordea keeps it in her chambers. It has a golden hilt and a serpentine blade.",
            },
            Stage {
                when: "lothianDead",
                de: "Ich habe Vividos den Dolch gebracht. In der Ewigkeitszeremonie schickte er Lothian damit zu Lithos. Nun ist Vividos selbst der Nekromant.",
                en: "I brought Vividos the dagger. In the Ceremony of Eternity he sent Lothian to Lithos with it. Now Vividos himself is the necromancer.",
            },
        ],
    },
    Quest {
        de: "Lehrling der Nekromanten",
        en: "Apprentice of the necromancers",
        done: "has:442:1",
        stages: &[
            Stage {
                when: "isApprentice",
                de: "Vividos hat mich als Lehrling angenommen. Die Bibliothek im oberen Stockwerk des Mausoleums steht mir offen.",
                en: "Vividos has taken me as his apprentice. The library on the upper floor of the mausoleum is open to me.",
            },
            Stage {
                when: "toldReagents",
                de: "Ich soll Vividos zwei Reagenzien bringen: eine Henkersmaske, eine schwarze Pflanze mit vielen Blättern, die unter einem großen Baum zwischen Tenebrae und dem Friedhof wächst, und die Ellbogen des Toten, gefallene Stöcke aus Westtenebrae, wo Geister umherirren.",
                en: "I am to bring Vividos two reagents: executioner's hood, a black plant with many leaves that grows under a large tree between Tenebrae and the cemetery, and deadman's elbows, fallen sticks from West Tenebrae, where ghosts wander.",
            },
            Stage {
                when: "toldReagents & mythSticks",
                de: "Mythran hat Gerüchte über einen Kreis toter Bäume auf dem Plateau gehört.",
                en: "Mythran has heard rumours of a circle of dead trees on the plateau.",
            },
            Stage {
                when: "haveReagentsDone",
                de: "Vividos gab mir den Schlüssel des Berghüters. Mit ihm und dem Zauber 'Offene Erde' öffne ich den Eingang zu den Oberen Katakomben. Dort soll ich mit dem Seancezauber die alten Nekromanten befragen. Beide Zauber finde ich in der Bibliothek; die Reagenzien gehören in einen Beutel.",
                en: "Vividos gave me the Key of the Caretaker. With it and the 'Open Ground' spell I can open the way into the Upper Catacombs. There I am to question the ancient necromancers with the seance spell. Both spells are in the library; the reagents go into a pouch.",
            },
            Stage {
                when: "metNec1|metNec2|metNec3|metNec4|metNec5|metNec6",
                de: "In den Katakomben habe ich mit den Geistern der alten Nekromanten gesprochen.",
                en: "In the catacombs I have spoken with the spirits of the ancient necromancers.",
            },
            Stage {
                when: "lithosMet",
                de: "Ich bin vor den Thron von Lithos, dem Bergkönig, getreten. Um sein Diener zu werden, muß ich Lothian in seiner Erde begraben. Wie das geht, soll mir der Nekromant sagen.",
                en: "I stood before the throne of Lithos, the Mountain King. To become his servant I must bury Lothian in his earth. The necromancer is to teach me how.",
            },
            Stage {
                when: "gotTheKey",
                de: "Vividos gab mir den Novizenschlüssel. Ich muß Lothians Leichnam damit berühren. Er liegt außerhalb des Mausoleums, in der nordöstlichen Ecke der Mauern.",
                en: "Vividos gave me the Key of the Scion. I must touch Lothian's body with it. It lies outside the mausoleum, in the northeast corner of the walls.",
            },
            Stage {
                when: "lothInterred",
                de: "Ich habe Lothian begraben.",
                en: "I have buried Lothian.",
            },
            Stage {
                when: "saidBirth",
                de: "Vividos hat mich zu seinem Ersten Schüler ernannt. Nun soll ich zu Moriens' Geburtsort pilgern und um eine Vision bitten. Der Weg führt erneut durch die Katakomben; Schilder weisen ihn.",
                en: "Vividos has named me his Scion. Now I am to make a pilgrimage to the birthplace of Moriens and ask for a vision. The way leads through the catacombs again; signs point the way.",
            },
            Stage {
                when: "openBirth",
                de: "Ich habe den Geburtsort von Moriens erreicht.",
                en: "I have reached the birthplace of Moriens.",
            },
            Stage {
                when: "postEarthQuake",
                de: "Mit dem Schädel der Beben habe ich ein Erdbeben ausgelöst. In den Katakomben haben sich neue Wege aufgetan.",
                en: "With the Skull of Quakes I caused an earthquake. New ways have opened in the catacombs.",
            },
            Stage {
                when: "has:442:1",
                de: "Moriens hatte das Herz der Erde mit in die Stadt der Toten genommen. Nun habe ich es an mich genommen.",
                en: "Moriens had taken the Heart of Earth with him into the City of the Dead. Now I have taken it.",
            },
        ],
    },
    Quest {
        de: "Devon im Kerker",
        en: "Devon imprisoned",
        done: "devonInRule",
        stages: &[
            Stage {
                when: "metDevonInJail",
                de: "Devon sitzt im Gefängnis, und niemand sagt ihm, warum. Sein Freund, der Gelehrte Bentic, wurde hingerichtet. Devon bat mich herauszufinden, was Bentic verbrochen hat. Salkind, Mordeas Majordomus, sollte es wissen.",
                en: "Devon is in prison, and nobody will tell him why. His friend, the scholar Bentic, has been executed. Devon asked me to find out what Bentic's crime was. Salkind, Mordea's majordomo, should know.",
            },
            Stage {
                when: "refusedDevon & !helpDevon",
                de: "Ich habe Devon meine Hilfe verweigert.",
                en: "I refused to help Devon.",
            },
            Stage {
                when: "helpDevon",
                de: "Ich habe Devon versprochen, ihm zu helfen.",
                en: "I promised to help Devon.",
            },
            Stage {
                when: "secondexecution|devonInRule",
                de: "Man hat mich mit Bentics Tagebuch ertappt und zum Richtplatz am Steg gebracht, wo auch Devon hingerichtet werden sollte.",
                en: "I was caught with Bentic's journal and taken to the place of execution on the dock, where Devon too was to die.",
            },
            Stage {
                when: "devonInRule",
                de: "Am Steg forderte Devon Mordea heraus: Auch er trägt die Gezeitenmacht in sich. Er besiegte sie, und Mordeas Tyrannei ist zu Ende. Devon wird als neuer Gezeitenfürst über Tenebrae herrschen.",
                en: "On the dock Devon challenged Mordea: he too carries the power of the tides. He defeated her, and Mordea's tyranny is over. Devon will rule Tenebrae as the new Tempest.",
            },
        ],
    },
    Quest {
        de: "Der Geist des Störtebeker",
        en: "The ghost of Stoertebeker",
        done: "readGravestone",
        stages: &[
            Stage {
                when: "toldGhostStory",
                de: "Orlok erzählte von seinem Ahnen Klaus Störtebeker. Der fand auf See den Schatz der alten Könige, wurde dann aber von Alpträumen heimgesucht. Er vergrub den Schatz in den Hügeln und kehrte nie zurück. Sein Geist soll noch heute dort umgehen.",
                en: "Orlok told of his ancestor Klaus Stoertebeker, who found the treasure of the old kings at sea but was then haunted by nightmares. He buried the treasure in the hills and never returned. His ghost is said to wander there still.",
            },
            Stage {
                when: "gwillimToldDoor",
                de: "Gwillim warnte mich vor dem Geist: Viele sind bei der Suche gestorben, auch sein Bruder. Nur Kilandra überlebte, verlor aber den Verstand. Sie soll einen Schlüssel haben, mit dem man zum Geist gelangt. Sie wohnt am Meer.",
                en: "Gwillim warned me about the ghost: many died searching for it, his brother among them. Only Kilandra survived, but she lost her mind. She is said to have a key that leads to the ghost. She lives by the sea.",
            },
            Stage {
                when: "toldAboutKey",
                de: "Kilandra sagt, ihre tote Tochter habe den Schlüssel. Bei ihr im Grab sei er sicher.",
                en: "Kilandra says her dead daughter has the key. It is safe with her in the grave.",
            },
            Stage {
                when: "readGravestone",
                de: "Am Grab von Kilandras Tochter Elena erschien ihr Geist und gab mir den Schlüssel.",
                en: "At the grave of Kilandra's daughter Elena her ghost appeared and gave me the key.",
            },
        ],
    },
    Quest {
        de: "Der Orden der Erleuchtung",
        en: "The Order of Enlightenment",
        done: "has:442:3",
        stages: &[
            Stage {
                when: "xavierMet|stellosMet|cyrrusMet|torwinMet",
                de: "Ich bin auf der Insel Silberfels. Hier leben die Theurgisten, die Jünger der Lufttitanin Stratos. Sie nutzen einen kleinen Teil ihrer Macht, um zu heilen. Ihr Oberhaupt ist der blinde Stellos.",
                en: "I am on Argentrock Isle. Here live the theurgists, the disciples of Stratos, the Titan of Air. They use a small part of her power to heal. Their leader is the blind Stellos.",
            },
            Stage {
                when: "spokeOfTests|readyForTests|attemptedWisdomTest|passedWisdomTest",
                de: "Wer in den Orden der Erleuchtung eintreten will, muß Prüfungen bestehen. Bruder Xavier nimmt die ersten beiden ab: die Prüfung der Weisheit, fünf Fragen über das Leben der Theurgisten, und die Prüfung des Gleichgewichts. Die Bücher in der Klosterbibliothek helfen bei den Fragen.",
                en: "Whoever wishes to join the Order of Enlightenment must pass tests. Brother Xavier holds the first two: the Test of Wisdom, five questions about the life of the theurgists, and the Test of Centeredness. The books in the monastery library help with the questions.",
            },
            Stage {
                when: "passedWisdomTest",
                de: "Ich habe die Prüfung der Weisheit bestanden. Für die Prüfung des Gleichgewichts muß ich zur Sturmspitze und auf den höchsten Felsen klettern. Dort muß ich den Winden standhalten.",
                en: "I have passed the Test of Wisdom. For the Test of Centeredness I must go to Stormpeak and climb the highest rock. There I must withstand the winds.",
            },
            Stage {
                when: "passedCenteredTest",
                de: "Ich habe die Prüfung des Gleichgewichts bestanden und darf mich Klosterschüler nennen. Nun soll ich den Ehrenwerten Stellos aufsuchen.",
                en: "I have passed the Test of Centeredness and may call myself an Initiate. Now I am to seek out the Honourable Stellos.",
            },
            Stage {
                when: "silverMineKeyGiven",
                de: "Ich muß meine eigenen Zaubersymbole anfertigen. Stellos gab mir den Schlüssel zur Silbermine in den Höhlen unter dem Kloster. Das Silber bringe ich zum Schmied Korick in Tenebrae; die fertigen Symbole lege ich auf den Symbolaltar im Kloster.",
                en: "I must make my own foci. Stellos gave me the key to the silver mine in the caves beneath the monastery. I take the silver to the smith Korick in Tenebrae and lay the finished foci on the Altar of Focus in the monastery and call upon Stratos.",
            },
            Stage {
                when: "hasOwnFoci",
                de: "Meine Zaubersymbole sind fertig.",
                en: "My foci are finished.",
            },
            Stage {
                when: "madeToraxForTest",
                de: "Für die dritte Prüfung muß ich erneut in die Höhlen unter dem Kloster. Sie verlangt alles, was ich bisher gelernt habe.",
                en: "For the third test I must go into the caves beneath the monastery again. It demands everything I have learned so far.",
            },
            Stage {
                when: "passedThirdTest",
                de: "Ich habe die dritte Prüfung bestanden.",
                en: "I have passed the third test.",
            },
            Stage {
                when: "focusMissing",
                de: "Bruder Xaviers Heilsymbol ist verschwunden, wahrscheinlich gestohlen. Bis es wieder da ist, ruht meine Ausbildung.",
                en: "Brother Xavier's healing focus has vanished, probably stolen. Until it is found, my training rests.",
            },
            Stage {
                when: "solvedMystery",
                de: "Der Schüler Torwin hatte das Heilsymbol genommen. Er ist der Sohn Torans, der in Tenebrae hingerichtet wurde, und wollte seinen Vater mit Stratos' Magie aus der Stadt der Toten zurückholen. Dann stürzte er sich in den Tod. Das Symbol muß zu Xavier zurück.",
                en: "The student Torwin had taken the healing focus. He is the son of Toran, who was executed in Tenebrae, and wanted to bring his father back from the City of the Dead with Stratos' magic. Then he leapt to his death. The focus must go back to Xavier.",
            },
            Stage {
                when: "fociReturned",
                de: "Ich habe Xavier sein Heilsymbol zurückgebracht.",
                en: "I returned Xavier's healing focus to him.",
            },
            Stage {
                when: "permissionToAirWalk",
                de: "Für die letzte Prüfung soll ich zur Sturmspitze gehen und mich zwischen die zwei schwarzen Säulen stellen. Um die Mystische Stimme zu hören, muß ich mich ganz auf meinen Glauben verlassen.",
                en: "For the final test I am to go to Stormpeak and stand between the two black pillars. To hear the Mystic Voice I must rely wholly on my faith.",
            },
            Stage {
                when: "stratosMet",
                de: "Ich bin Stratos begegnet, der Titanin der Lüfte. Durch den Atem des Windes heilen ihre auserwählten Kinder. Nur Stellos kennt sein Versteck, und Stratos will ihn nicht hergeben.",
                en: "I met Stratos, the Titan of Air. Through the Breath of Wind her chosen children heal. Only Stellos knows where it is hidden, and Stratos will not give it up.",
            },
            Stage {
                when: "has:442:3",
                de: "Ich habe den Atem des Windes an mich genommen. Damit ist das Band zwischen Stratos und ihren Heilern zerrissen.",
                en: "I have taken the Breath of Wind. With it the bond between Stratos and her healers is broken.",
            },
        ],
    },
    Quest {
        de: "Torwins Ring",
        en: "Torwin's ring",
        done: "rhiangrieving",
        stages: &[
            Stage {
                when: "has:832 & torwinJumped",
                de: "Ich habe Torwins Ring gefunden, ein Geschenk seines Vaters. Er sollte zu seiner Mutter Rhian, einer Weberin in Tenebrae.",
                en: "I found Torwin's ring, a gift from his father. It should go to his mother Rhian, a weaver in Tenebrae.",
            },
            Stage {
                when: "rhiangrieving",
                de: "Ich habe Rhian den Ring ihres Sohnes gebracht.",
                en: "I brought Rhian her son's ring.",
            },
        ],
    },
    Quest {
        de: "Die Wassertitanin",
        en: "The Titan of Water",
        done: "hydrosFree",
        stages: &[
            Stage {
                when: "metHydros",
                de: "Ich bin Hydros begegnet, der Wassertitanin. Sie ist gefangen.",
                en: "I met Hydros, the Titan of Water. She is imprisoned.",
            },
            Stage {
                when: "hydrosBegged",
                de: "Hydros bat mich, zur Grabstätte der Geliebten jenes Mannes zu gehen, der sie gefangen hält, und dort den Boden zu öffnen, damit das Wasser auf die Klippen fließen kann. Dann wäre sie frei.",
                en: "Hydros asked me to open the ground next to the gravestone of her captor's beloved, releasing the waters on the cliffside. Then she will be free.",
            },
            Stage {
                when: "hydrosFree",
                de: "Ich habe den Boden geöffnet, und Hydros ist frei.",
                en: "I opened the ground, and Hydros is free.",
            },
            Stage {
                when: "hydrosFree=2",
                de: "Hydros will sich an den Nachkommen ihres Kerkermeisters rächen. Sie ist auf dem Weg zu Devon!",
                en: "Hydros wants revenge on the descendants of her jailer. She is on her way to Devon!",
            },
        ],
    },
    Quest {
        de: "Die Träne der Meere",
        en: "The Tear of the Seas",
        done: "has:442:2",
        stages: &[
            Stage {
                when: "devonMentionedTear",
                de: "Devon fand vor Jahren einen großen schwarzen Stein in seinen Netzen, härter als Obsidian. Er hat ihn noch in einer Truhe in der südwestlichen Ecke seines Arbeitszimmers und gab mir den Schlüssel.",
                en: "Years ago Devon found a large black stone in his nets, harder than obsidian. He still keeps it in a chest in the southwest corner of his study and gave me the key.",
            },
            Stage {
                when: "has:442:2",
                de: "Ich habe die Träne der Meere.",
                en: "I have the Tear of the Seas.",
            },
        ],
    },
    Quest {
        de: "Die Hexer",
        en: "The sorcerers",
        done: "has:442:4",
        stages: &[
            Stage {
                when: "vardionEnclave|baneMet|vardionMet|gorgrondMet|berenMet",
                de: "Auf der Dämonenklippe im Schatten des Vulkans Morgaelin liegt die Enklave der Hexer. Sie beherrschen das Feuer und halten Pyros, den Herrn der Flamme, gebunden. Ihr Meister ist Malchir.",
                en: "On Daemon's Crag in the shadow of the volcano Morgaelin lies the enclave of the sorcerers. They command fire and keep Pyros, the Lord of Flame, bound. Their master is Malchir.",
            },
            Stage {
                when: "baneHasAsked",
                de: "Die Akoluthin Bane glaubt, daß der Oberste Akoluth Vardion den Meister ermorden will. Sie will meine Sponsorin werden, wenn ich Vardions Wahrnamen für sie herausfinde.",
                en: "The acolyte Bane believes that the Chief Acolyte Vardion plans to murder the master. She will become my sponsor if I find out Vardion's Truename for her.",
            },
            Stage {
                when: "baneJoined & !vardionJoined",
                de: "Ich bin Banes Novize. Mein Wahrname ist Daemion, ihrer Sabriane. Ich soll Vardions Vertrauen gewinnen und seinen Wahrnamen herausfinden.",
                en: "I am Bane's novice. My Truename is Daemion, hers is Sabriane. I am to win Vardion's trust and find out his Truename.",
            },
            Stage {
                when: "vardionJoined & !baneJoined",
                de: "Ich bin Vardions Novize. Mein Wahrname ist Daemion, seiner Morduin. Er glaubt, Bane wolle ihn und den Meister aus dem Hinterhalt ermorden. Ich soll ihren Wahrnamen herausfinden.",
                en: "I am Vardion's novice. My Truename is Daemion, his is Morduin. He believes Bane means to murder him and the master from ambush. I am to find out her Truename.",
            },
            Stage {
                when: "acolyteKilled=1",
                de: "Ich verriet Vardion Banes Wahrnamen, und er hat sie vernichtet. Malchir nahm mich als Novizen auf. In der Bibliothek der Enklave lerne ich, was ein Hexer wissen muß.",
                en: "I told Vardion Bane's Truename, and he destroyed her. Malchir accepted me as a novice. In the library of the enclave I can learn what a sorcerer must know.",
            },
            Stage {
                when: "acolyteKilled=2",
                de: "Ich verriet Bane Vardions Wahrnamen, und sie hat ihn vernichtet. Malchir nahm mich als Novizen auf. In der Bibliothek der Enklave lerne ich, was ein Hexer wissen muß.",
                en: "I told Bane Vardion's Truename, and she destroyed him. Malchir accepted me as a novice. In the library of the enclave I can learn what a sorcerer must know.",
            },
            Stage {
                when: "sponsorTest|keyOfTheAcolyte",
                de: "Erste Prüfung: Am Pentagramm meines Sponsors muß ich Symbole mit den Zaubern 'Funkenflug', 'Feuerkugel' und 'Hitzeschild' verzaubern.",
                en: "First test: at my sponsor's pentagram I must enchant foci with the spells 'Flash', 'Flame Bolt' and 'Endure Heat'.",
            },
            Stage {
                when: "keyOfTheAcolyte",
                de: "Ich habe die erste Prüfung bestanden und den Schlüssel des Akoluthen erhalten: die Fähigkeit, die Obsidianfestung zu betreten.",
                en: "I passed the first test and received the Key of the Acolyte: the ability to enter the Obsidian Fortress.",
            },
            Stage {
                when: "arcadionMet",
                de: "In der Obsidianfestung traf ich den Dämonenhexer Arcadion. Erst wenn ich die Prüfungen der Festung bestanden habe, darf ich durch das Tor zu Malchirs heiligen Gemächern.",
                en: "In the Obsidian Fortress I met the daemon sorcerer Arcadion. Only when I have passed the trials of the fortress may I go through the gate to Malchir's sacred chambers.",
            },
            Stage {
                when: "goToInnerSanctum",
                de: "Ich habe die Prüfungen der Obsidianfestung bestanden. Malchir erwartet mich in seinen Gemächern.",
                en: "I have passed the trials of the Obsidian Fortress. Malchir awaits me in his chambers.",
            },
            Stage {
                when: "MalchirTest=6",
                de: "Ich habe Malchirs Prüfung bestanden. Zuletzt muß ich mich im Ritual des Feuers bewähren, in dem die Hexer Pyros anrufen: eine rote Kerze mitbringen, sie mit einem Zauber entzünden, niederknien und 'In Sanct An Por' sprechen.",
                en: "I have passed Malchir's test. Last I must prove myself in the ritual of fire, in which the sorcerers call upon Pyros: bring a red candle, light it with a spell, kneel and speak 'In Sanct An Por'.",
            },
            Stage {
                // (the flag "avatarMasterSorcerer" only means that the
                // sorcerers have turned against him: fighting one sets it)
                when: "inDaemonScene & dead:27",
                de: "Nach dem Ritual griff Malchir mich an. Ich habe ihn besiegt und bin nun selbst der Meisterhexer.",
                en: "After the ritual Malchir attacked me. I defeated him and am now the Master Sorcerer myself.",
            },
            Stage {
                when: "has:442:4",
                de: "Ich habe die Flammenzunge an mich genommen, mit der die Hexer Pyros gebunden halten.",
                en: "I have taken the Tongue of Flame, with which the sorcerers keep Pyros bound.",
            },
        ],
    },
];
