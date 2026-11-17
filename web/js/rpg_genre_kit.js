// =============================================================================
// NYX ENGINE — RPG GENRE KIT (STATS, INVENTORY, COMBAT, QUESTS)
// =============================================================================

const NyxRPGKit = {
  // Player Stats Container
  stats: {
    level: 1,
    xp: 0,
    xpToNextLevel: 100,
    statPoints: 5,

    // Base Attributes
    base: {
      str: 14, // Strength -> Physical ATK
      dex: 12, // Dexterity -> Crit Chance & Move Speed
      int: 10, // Intelligence -> Magic ATK & Max MP
      vit: 15, // Vitality -> Max HP & Defense
      luk: 8   // Luck -> Drop Rate & Crit Multiplier
    },

    // Derived Attributes (Recalculated via Pipeline)
    derived: {
      maxHp: 150,
      currentHp: 150,
      maxMp: 80,
      currentMp: 80,
      stamina: 100,
      atk: 32,
      def: 18,
      critChance: 5.4, // %
      critDamage: 1.5, // 150%
      moveSpeed: 3.8
    },

    // Active Buffs & Debuffs
    statusEffects: []
  },

  // Equipment Sockets
  equipment: {
    weapon: { id: 'iron_sword', name: 'Iron Broadsword +1', type: 'weapon', atk: 12, rarity: 'rare' },
    shield: { id: 'wooden_shield', name: 'Oak Round Shield', type: 'shield', def: 6, rarity: 'common' },
    armor: { id: 'leather_tunic', name: 'Reinforced Leather Armor', type: 'armor', def: 10, rarity: 'common' },
    helmet: null,
    ring: { id: 'ruby_ring', name: 'Ring of Minor Might', type: 'ring', bonusStr: 3, rarity: 'rare' }
  },

  // Grid Inventory Items
  inventory: [
    { id: 'hp_potion', name: 'Health Potion', count: 5, type: 'consumable', heal: 50, rarity: 'common' },
    { id: 'mp_potion', name: 'Mana Potion', count: 3, type: 'consumable', mana: 40, rarity: 'common' },
    { id: 'iron_ingot', name: 'Iron Ingot', count: 12, type: 'material', rarity: 'common' },
    { id: 'ancient_tome', name: 'Tome of Abyssal Secrets', count: 1, type: 'quest', rarity: 'legendary' }
  ],

  // Quest State Machine
  quests: {
    'q_titanic_01': {
      id: 'q_titanic_01',
      title: 'Echoes of 11:40 PM',
      desc: 'Investigate the Marconi Radio Room and retrieve Captain Smith emergency telegram.',
      status: 'in_progress', // not_started, in_progress, turn_in, completed
      objectives: [
        { text: 'Locate Stateroom Door B52', completed: true },
        { text: 'Recover Marconi Telegram 1912', completed: false }
      ],
      rewardXp: 250,
      rewardGold: 100
    }
  },

  // Initialize RPG System
  init(scene, camera) {
    this.scene = scene;
    this.camera = camera;
    this.recalculateStats();
    console.log('✔ [RPGKit] RPG Genre Kit (Stats, Inventory, Quests, Combat) initialized.');
  },

  // Recalculates stats through the 3-stage modifier pipeline
  recalculateStats() {
    const b = this.stats.base;
    let flatStr = 0, flatDef = 0, flatAtk = 0;

    // Sum flat bonuses from equipment
    Object.values(this.equipment).forEach(item => {
      if (!item) return;
      if (item.atk) flatAtk += item.atk;
      if (item.def) flatDef += item.def;
      if (item.bonusStr) flatStr += item.bonusStr;
    });

    const finalStr = b.str + flatStr;
    const finalVit = b.vit;
    const finalDex = b.dex;
    const finalInt = b.int;

    this.stats.derived.maxHp = finalVit * 10;
    this.stats.derived.maxMp = finalInt * 8;
    this.stats.derived.atk = (finalStr * 2) + flatAtk;
    this.stats.derived.def = finalVit + flatDef;
    this.stats.derived.critChance = Number((finalDex * 0.45).toFixed(1));
    this.stats.derived.moveSpeed = 3.6 + (finalDex * 0.05);

    // Clamp current values
    this.stats.derived.currentHp = Math.min(this.stats.derived.currentHp, this.stats.derived.maxHp);
    this.stats.derived.currentMp = Math.min(this.stats.derived.currentMp, this.stats.derived.maxMp);

    return this.stats.derived;
  },

  // Add Experience & Level Up Pipeline
  gainXp(amount) {
    this.stats.xp += amount;
    console.log(`[RPG] Gained +${amount} XP (${this.stats.xp}/${this.stats.xpToNextLevel})`);

    while (this.stats.xp >= this.stats.xpToNextLevel) {
      this.stats.xp -= this.stats.xpToNextLevel;
      this.stats.level++;
      this.stats.statPoints += 5;
      this.stats.xpToNextLevel = Math.floor(this.stats.xpToNextLevel * 1.5);

      // Restore full HP/MP on level up
      this.recalculateStats();
      this.stats.derived.currentHp = this.stats.derived.maxHp;
      this.stats.derived.currentMp = this.stats.derived.maxMp;

      if (window.showEditorNotification) {
        window.showEditorNotification(`LEVEL UP! You are now Level ${this.stats.level}! (+5 Stat Points)`);
      }
      console.log(`🌟 [RPG Level Up] Advanced to Level ${this.stats.level}!`);
    }
  },

  // Apply Buff or Debuff
  applyStatusEffect(effectName, durationSec = 5.0, value = 10) {
    const effect = {
      name: effectName, // 'burn', 'poison', 'freeze', 'haste'
      duration: durationSec,
      value: value,
      startTime: performance.now(),
      lastTick: performance.now()
    };
    this.stats.statusEffects.push(effect);
    console.log(`[RPG Effect] Applied ${effectName} for ${durationSec}s`);
  },

  // Melee Combat Attack (Swept Hitbox Window & Hit-Stop Impact)
  executeMeleeAttack(attackerPos, facingDirection, attackRange = 2.4, sweepAngle = Math.PI / 2.5) {
    if (!this.scene) return [];

    console.log(`[RPG Combat] Executing melee slash (Range: ${attackRange}m)...`);

    const hitEnemies = [];
    const forward = facingDirection.clone().normalize();

    this.scene.traverse(obj => {
      if (obj.userData && obj.userData.isEnemy && obj.visible) {
        const toEnemy = obj.position.clone().sub(attackerPos);
        const dist = toEnemy.length();

        if (dist <= attackRange) {
          toEnemy.normalize();
          const dot = forward.dot(toEnemy);
          const angle = Math.acos(Math.max(-1, Math.min(1, dot)));

          if (angle <= sweepAngle / 2) {
            // Hit Enemy!
            const isCrit = Math.random() * 100 < this.stats.derived.critChance;
            const dmg = this.stats.derived.atk * (isCrit ? this.stats.derived.critDamage : 1.0);

            hitEnemies.push({
              enemy: obj,
              damage: dmg,
              isCrit: isCrit
            });

            console.log(`💥 [RPG Hit] Slashed "${obj.name}" for ${dmg.toFixed(0)} dmg ${isCrit ? '(CRITICAL HIT!)' : ''}`);
          }
        }
      }
    });

    return hitEnemies;
  },

  // Blacksmith Upgrade System (+1 to +10)
  upgradeWeapon(weaponKey = 'weapon') {
    const item = this.equipment[weaponKey];
    if (!item) return { success: false, msg: 'No weapon equipped' };

    const currentLevel = item.level || 1;
    if (currentLevel >= 10) return { success: false, msg: 'Max upgrade level reached (+10)' };

    // Cost: 3 Iron Ingots per level
    const costIngots = currentLevel * 2;
    const ingotItem = this.inventory.find(i => i.id === 'iron_ingot');
    if (!ingotItem || ingotItem.count < costIngots) {
      return { success: false, msg: `Requires ${costIngots}x Iron Ingots to upgrade.` };
    }

    ingotItem.count -= costIngots;
    item.level = currentLevel + 1;
    item.atk = Math.floor(item.atk * 1.25) + 3;
    item.name = item.name.replace(/\+\d+$/, '') + ` +${item.level}`;

    this.recalculateStats();
    console.log(`⚒️ [Blacksmith] Successfully upgraded ${item.name}! (New ATK: ${item.atk})`);
    return { success: true, item: item };
  },

  // Per-frame Status Effect Tick Update
  update(delta) {
    const now = performance.now();
    for (let i = this.stats.statusEffects.length - 1; i >= 0; i--) {
      const eff = this.stats.statusEffects[i];
      const elapsed = (now - eff.startTime) / 1000;

      // Tick periodic damage every 1.0s
      if (now - eff.lastTick >= 1000) {
        eff.lastTick = now;
        if (eff.name === 'burn' || eff.name === 'poison') {
          this.stats.derived.currentHp = Math.max(1, this.stats.derived.currentHp - eff.value);
          console.log(`🔥 [DoT Tick] ${eff.name} dealt -${eff.value} HP! (Current: ${this.stats.derived.currentHp}/${this.stats.derived.maxHp})`);
        }
      }

      if (elapsed >= eff.duration) {
        console.log(`[RPG Effect] ${eff.name} expired.`);
        this.stats.statusEffects.splice(i, 1);
      }
    }
  }
};

window.NyxRPGKit = NyxRPGKit;
console.log('✔ [Nyx Engine] RPG Genre Kit Loaded.');
