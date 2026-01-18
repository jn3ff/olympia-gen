This is going to be a side scrolling boss rush rogue like platformer called olympia. a "run" starts in a gladiator arena, and then directional decisions are made as a run's story evolves. runtime of a run i'm thinking 30min-2hrs the core loop is: platforming room (can be up, down, left, or right) where parkour is done, regular enemies are fought, directional decisions are made. at the end of the segment, a boss is fought, and a reward is gained. the reward will be tree style, with many different base trees available. it could also be a piece of equipment, or a base stat upgrade. i just say this to give you a sense of the types of knobs we want to be able to turn.  model movement off of hollow knight, except dash/roll should initially only be available on the ground. keep a mind to modularity - given the boss rush rogue nature of the game, levels and upgrades are hyper pluggable. 

equipment: - give armor (damage reduction and/or increased health) or grant new passives. Also, ownership of e.g. “the Helm of Darkness” can trigger future encounters, so we need some ability to tag equipment resources in a way that represents that.
Helmet
Chestplate
Greaves
Boots

Main hand
- Primary weapon
Categories - each weapon instance should have a baseline alignment with the categorical trait, and allow overrides for “light, heavy,special” strikes (hurtboxes, animations, etc). E.g. a sword with no overrides will use the base “sword” hurt boxes, cooldowns, etc, but maybe it’s a thrust-capable sword or something, so heavy gets overridden to its own sword moveset. 
1. Sword (e.g. Light: slash - thin hurt box, quick, medium damage. Heavy: Thrust, long thick hurt box which extends out, heavy damage. Special: Parry)
2. Spear (e.g. light: thrust - long thin hurt box, quick, medium damage. Heavy: thrust - long thin hurt box, slower, high damage. Special: Sweep, wide thick hurt box which extends out, heavy damage)
3. Bow (to be added - but not yet)
4. Daggers (to be added - but not yet)
5. Greatsword (to be added - but not yet)
6. Special (God weapon, hero weapon, etc, something with its entirely own unique moveset)

Money: 
- Can be spent at shops, or to trigger events, or to level up at shrines. Hermes will have a lot of progression around money & trade. Base system: enemies give a basic amount of coin reward. 


movement:

Single jump, ground dash, omnidirectional platforming. Double jump, triple jump, double dash, air dash, etc. can be unlocked in various ways from equipment or demigod progression.

Demigod system:

12 characters, one for each olympian. You start with a character who starts with a parental god. That god’s skill tree is immediately unlocked, and you can choose a blessing, stat improvement, or piece of equipment at the start of the run. The run always begins in the coliseum - maybe not THE coliseum, but a coliseum, a greek prisoner captured by the Romans and forced to fight in the gladiator arena. Boss blessings can be gained by visiting shrines for your god. Base tree system is one god-based, but you may encounter another god and become “champion” this will drive narrative events. If you become another god’s champion, an extremely powerful auxiliary tree is unlocked, all with blessings that synergize the powers of the two gods. The “skill” system will be founded in demigod upgrades - the quick character skill the character comes with can be swapped out for a different unlocked skill between rooms.

Character system:

12 characters, each one starts with a parental god. A character has some unique features.

1. Passive. Each character has their own passive which is fully unique & fully immutable
2. Skill. Each character has a rapid firing character skill (like a cast, or regen, or heavy damage, etc) which synergies & builds into their parental god’s skill tree
3. Ultimate (starts empty) - filled in by parental god. Narratively there will be opportunities to maybe fill this slot with something else, so it should be kept flexible, but green path “neutral neutral” alignment play will result in the parental god’s being the ultimate]
4. Starting equipment per character: characters will specialize in certain weapons. This will be reaffirmed by the choices available. But nothing stops a character from using whatever weapon they find along the way (we need to tune appearance rates & give min/max knobs to ensure e.g. a sword-base character always sees at least 1 sword but no more than 2 out of a weapon reward). The weapon they start with (if they don’t take the equipment reward) will be some basic version of their primary weapon class

The character sprite creation should be very simple & fully customizable (hair, skin tone, build, eye color). They’ll always start wearing rags to begin.

Rewards:

There are 
- Blessing rewards(found at shrines)
- Equipment rewards (found in chests - minor, or forges - major)
    - Minor chest rewards offer randomly, but out of 3 at least one armor piece and one weapon piece shown (with base rate tunable, and min/max as stated above configurable)
    - Forge Equipment rewards offer first a choice: weapon or armor, and then give a powerful version. Increased rarity odds, increased odds you get a piece for a slot you need, higher drop rate for special. 
- Stat rewards (found at shrines) - pay money to level up, or certain blessings will modify cost/give free stat boosts.
- Shops
    - found in central hubs, with the occasional wandering trader available. 
    - Spend money on new equipment
    - See blacksmith to improve current equipment
    - See mage to enchant current equipment

The RogueLike Twist:

Narratively, you’re kind of playing the game as the GODS. At the end of the run (demigod dies or succeeds), narrative unfolding occurs from the gods POV, and you gain some resource from the performance of your child. (Call this currency “faith” for now until a better title unfolds. Here, you invest in your child -  add favor (strengthen blessing tree/improve encounter rates), improve base stats, add stronger starting equipment, build relationships with other gods (unlocking & improving cross-god encounter rates), purchase powerful “chosen one” single-time buffs for the child’s next run. The actual narrative thrust on the game is that you are playing and experiencing events ultimately through the Gods’ perspective, and their children mostly die in heroic failure. 

Loose Idea notes:

- Ascension to godhood. When a character wins, you can pay as godly parent to ascend them if you like. This means they will hang around Olympus, and unlock a new specialty fight (player has to beat the Ascended version of their prior character)
- Random buffs from speciality encounters can be tacked on ad infinitum. E.g. Prometheus, any boss that blesses you, etc. these don't play with the blessing tree, but can add any sort of passive improvement imaginable (AOE, health, stat, equipment, etc.)
