// =============================================================================
// NYX ENGINE — FPS / SHOOTER GENRE KIT (CONTROLLER, WEAPONS, BALLISTICS)
// =============================================================================

const NyxFPSKit = {
  // Weapon Presets
  WEAPONS: {
    rifle: {
      name: 'Nyx AR-47 (Assault Rifle)',
      type: 'rifle',
      fireMode: 'auto', // semi, burst, auto
      fireRate: 650, // RPM
      damage: 28,
      magSize: 30,
      reserveAmmo: 120,
      currentMag: 30,
      reloadTime: 2.1,
      recoilVertical: 0.035,
      recoilHorizontal: 0.012,
      spread: 0.015,
      muzzleVelocity: 720, // m/s
      isHitscan: true,
      adsFov: 48
    },
    shotgun: {
      name: 'Breacher-12 (Pump Shotgun)',
      type: 'shotgun',
      fireMode: 'semi',
      fireRate: 85,
      damage: 14, // per pellet
      pellets: 8,
      magSize: 8,
      reserveAmmo: 32,
      currentMag: 8,
      reloadTime: 3.2,
      recoilVertical: 0.095,
      recoilHorizontal: 0.025,
      spread: 0.065,
      muzzleVelocity: 410,
      isHitscan: true,
      adsFov: 55
    },
    sniper: {
      name: 'Vanguard .338 (Bolt Sniper)',
      type: 'sniper',
      fireMode: 'semi',
      fireRate: 45,
      damage: 115,
      magSize: 5,
      reserveAmmo: 20,
      currentMag: 5,
      reloadTime: 3.6,
      recoilVertical: 0.14,
      recoilHorizontal: 0.01,
      spread: 0.001,
      muzzleVelocity: 940,
      isHitscan: false, // Physical projectile with bullet drop!
      adsFov: 24
    }
  },

  // Active FPS State
  state: {
    activeWeaponKey: 'rifle',
    isAiming: false,
    isReloading: false,
    isFiring: false,
    lastFireTime: 0,
    burstCount: 0,
    leanAngle: 0.0, // Q / E leaning (-15 deg to +15 deg)
    targetLeanAngle: 0.0,
    headBobTimer: 0.0,
    cameraShakeAmount: 0.0,
    activeBullets: [], // Physical projectile ballistics
    decals: []
  },

  // Initialize or Attach FPS Controller to Camera
  initController(camera, scene) {
    this.camera = camera;
    this.scene = scene;
    this.activeWeapon = { ...this.WEAPONS.rifle };

    // Build or attach 3D Viewmodel Gun
    this.buildWeaponViewmodel();
    console.log('✔ [FPSKit] First-Person Controller & Weapon System initialized.');
  },

  // Procedural 3D Weapon Viewmodel attached to camera
  buildWeaponViewmodel() {
    if (!this.camera) return;

    let vm = this.camera.getObjectByName('FPS_Viewmodel_Gun');
    if (vm) this.camera.remove(vm);

    const gunGroup = new THREE.Group();
    gunGroup.name = 'FPS_Viewmodel_Gun';

    // Barrel
    const barrelGeo = new THREE.BoxGeometry(0.06, 0.08, 0.55);
    const metalMat = new THREE.MeshStandardMaterial({ color: 0x1f2428, roughness: 0.35, metalness: 0.85 });
    const barrel = new THREE.Mesh(barrelGeo, metalMat);
    barrel.position.set(0, 0, -0.25);
    gunGroup.add(barrel);

    // Receiver / Body
    const bodyGeo = new THREE.BoxGeometry(0.08, 0.12, 0.28);
    const darkMat = new THREE.MeshStandardMaterial({ color: 0x111315, roughness: 0.5, metalness: 0.6 });
    const body = new THREE.Mesh(bodyGeo, darkMat);
    body.position.set(0, -0.04, 0.05);
    gunGroup.add(body);

    // Magazine
    const magGeo = new THREE.BoxGeometry(0.05, 0.2, 0.08);
    const mag = new THREE.Mesh(magGeo, darkMat);
    mag.position.set(0, -0.16, 0.02);
    mag.rotation.x = -0.2;
    gunGroup.add(mag);

    // Sight / Scope Socket
    const sightGeo = new THREE.BoxGeometry(0.03, 0.04, 0.06);
    const sightMat = new THREE.MeshStandardMaterial({ color: 0x00f0ff, emissive: 0x00a0cc, emissiveIntensity: 0.5 });
    const sight = new THREE.Mesh(sightGeo, sightMat);
    sight.position.set(0, 0.06, -0.1);
    gunGroup.add(sight);

    // Default Hip Position (bottom right of camera)
    gunGroup.position.set(0.24, -0.22, -0.45);
    this.hipPos = new THREE.Vector3(0.24, -0.22, -0.45);
    this.adsPos = new THREE.Vector3(0.0, -0.12, -0.32); // Centered ADS
    this.gunGroup = gunGroup;

    this.camera.add(gunGroup);
    return gunGroup;
  },

  // Fire Weapon Trigger
  fire(raycaster, crosshairEl) {
    if (!this.activeWeapon || this.state.isReloading) return;
    if (this.activeWeapon.currentMag <= 0) {
      this.reload();
      return;
    }

    const now = performance.now();
    const fireInterval = (60 / this.activeWeapon.fireRate) * 1000;
    if (now - this.state.lastFireTime < fireInterval) return;

    this.state.lastFireTime = now;
    this.activeWeapon.currentMag--;

    // 1. Procedural Viewmodel Recoil Kick
    if (this.gunGroup) {
      this.gunGroup.position.z += 0.08;
      this.gunGroup.rotation.x += this.activeWeapon.recoilVertical;
      this.gunGroup.rotation.y += (Math.random() - 0.5) * this.activeWeapon.recoilHorizontal;
    }

    // 2. Camera Shake
    this.state.cameraShakeAmount = 0.035;

    // 3. Audio & Muzzle Flash effect
    if (window.NyxAudioSynth) {
      this.playGunshotSound(this.activeWeapon.type);
    }

    // 4. Ballistics / Hitscan Execution
    if (this.activeWeapon.isHitscan) {
      this.executeHitscan(raycaster);
    } else {
      this.spawnPhysicalProjectile();
    }

    console.log(`[FPS] Fired ${this.activeWeapon.name} | Mag: ${this.activeWeapon.currentMag}/${this.activeWeapon.magSize}`);
  },

  executeHitscan(raycaster) {
    if (!this.camera || !this.scene) return;

    // Calculate spread
    const spread = this.state.isAiming ? this.activeWeapon.spread * 0.3 : this.activeWeapon.spread;
    const spreadX = (Math.random() - 0.5) * spread;
    const spreadY = (Math.random() - 0.5) * spread;

    raycaster.setFromCamera(new THREE.Vector2(spreadX, spreadY), this.camera);

    const hitables = [];
    this.scene.traverse(obj => {
      if (obj.isMesh && obj.visible && obj !== this.gunGroup && obj.name !== 'Main_Camera_Helper') {
        hitables.push(obj);
      }
    });

    const hits = raycaster.intersectObjects(hitables, true);
    if (hits.length > 0) {
      const hit = hits[0];
      const targetObj = hit.object;

      // Check Damage Multiplier (Head vs Torso vs Limbs)
      let dmgMult = 1.0;
      let hitPart = 'Torso';
      if (targetObj.name.includes('Head') || hit.point.y > (targetObj.position.y + 1.2)) {
        dmgMult = 2.5;
        hitPart = 'Headshot!';
      } else if (targetObj.name.includes('Arm') || targetObj.name.includes('Leg')) {
        dmgMult = 0.75;
        hitPart = 'Limb';
      }

      const totalDmg = this.activeWeapon.damage * dmgMult;

      // Spawn Bullet Hole Decal
      this.spawnBulletImpact(hit.point, hit.face ? hit.face.normal : new THREE.Vector3(0, 1, 0));

      // Check Destructible Fracture
      if (targetObj.userData && targetObj.userData.isDestructible && window.NyxSmartComponents) {
        window.NyxSmartComponents.fractureEntity(targetObj, 8, totalDmg);
      }

      console.log(`[FPS Hit] Hit "${targetObj.name}" [${hitPart}] for ${totalDmg.toFixed(1)} damage!`);
    }
  },

  // Physical Ballistics Projectile (with gravity drop & velocity)
  spawnPhysicalProjectile() {
    if (!this.camera || !this.scene) return;

    const spawnPos = this.camera.position.clone();
    const forward = new THREE.Vector3(0, 0, -1).applyQuaternion(this.camera.quaternion);

    const tracerGeo = new THREE.SphereGeometry(0.04, 6, 6);
    const tracerMat = new THREE.MeshBasicMaterial({ color: 0xffe066 });
    const tracerMesh = new THREE.Mesh(tracerGeo, tracerMat);
    tracerMesh.position.copy(spawnPos);
    this.scene.add(tracerMesh);

    this.state.activeBullets.push({
      mesh: tracerMesh,
      position: spawnPos,
      velocity: forward.clone().multiplyScalar(this.activeWeapon.muzzleVelocity * 0.15), // scaled for simulation
      gravity: -9.81,
      lifetime: 3.0,
      spawnTime: performance.now()
    });
  },

  // Reload State Machine
  reload() {
    if (this.state.isReloading || this.activeWeapon.currentMag === this.activeWeapon.magSize) return;
    if (this.activeWeapon.reserveAmmo <= 0) return;

    this.state.isReloading = true;
    console.log(`[FPS] Reloading ${this.activeWeapon.name}...`);

    setTimeout(() => {
      const needed = this.activeWeapon.magSize - this.activeWeapon.currentMag;
      const amount = Math.min(needed, this.activeWeapon.reserveAmmo);
      this.activeWeapon.currentMag += amount;
      this.activeWeapon.reserveAmmo -= amount;
      this.state.isReloading = false;
      console.log(`[FPS] Reload complete! Mag: ${this.activeWeapon.currentMag}/${this.activeWeapon.magSize} (Reserve: ${this.activeWeapon.reserveAmmo})`);
    }, this.activeWeapon.reloadTime * 1000);
  },

  // Bullet Hole Decal on Wall
  spawnBulletImpact(hitPoint, normal) {
    if (!this.scene) return;

    const decalGeo = new THREE.CircleGeometry(0.05, 8);
    const decalMat = new THREE.MeshBasicMaterial({ color: 0x111111, side: THREE.DoubleSide });
    const decalMesh = new THREE.Mesh(decalGeo, decalMat);

    decalMesh.position.copy(hitPoint).addScaledVector(normal, 0.005);
    decalMesh.lookAt(hitPoint.clone().add(normal));

    this.scene.add(decalMesh);
    this.state.decals.push(decalMesh);

    // Limit decal count to 30
    if (this.state.decals.length > 30) {
      const old = this.state.decals.shift();
      if (old.parent) old.parent.remove(old);
    }
  },

  // Synthesize Gunshot Sound via Web Audio API
  playGunshotSound(type) {
    if (!window.NyxAudioSynth) return;
    const ctx = window.NyxAudioSynth.ctx;
    if (!ctx) return;

    const t = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = (type === 'shotgun') ? 'sawtooth' : 'triangle';
    osc.frequency.setValueAtTime(type === 'sniper' ? 320 : 220, t);
    osc.frequency.exponentialRampToValueAtTime(30, t + 0.18);

    gain.gain.setValueAtTime(0.4, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.22);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start(t);
    osc.stop(t + 0.22);
  },

  // Per-frame Animation Update for Viewmodel, Recoil Recovery, Leaning & Physical Ballistics
  update(delta) {
    if (!this.camera) return;

    // 1. Smooth ADS & Sway
    if (this.gunGroup) {
      const targetPos = this.state.isAiming ? this.adsPos : this.hipPos;
      this.gunGroup.position.lerp(targetPos, delta * 14.0);

      // Smooth Recoil Recovery
      this.gunGroup.rotation.x = THREE.MathUtils.lerp(this.gunGroup.rotation.x, 0, delta * 12.0);
      this.gunGroup.rotation.y = THREE.MathUtils.lerp(this.gunGroup.rotation.y, 0, delta * 12.0);
    }

    // 2. Corner Leaning (Q / E)
    this.state.leanAngle = THREE.MathUtils.lerp(this.state.leanAngle, this.state.targetLeanAngle, delta * 10.0);
    this.camera.rotation.z = this.state.leanAngle;

    // 3. Camera Shake recovery
    if (this.state.cameraShakeAmount > 0.001) {
      this.camera.position.x += (Math.random() - 0.5) * this.state.cameraShakeAmount;
      this.camera.position.y += (Math.random() - 0.5) * this.state.cameraShakeAmount;
      this.state.cameraShakeAmount *= 0.88;
    }

    // 4. Update Physical Ballistics (Projectiles)
    const now = performance.now();
    for (let i = this.state.activeBullets.length - 1; i >= 0; i--) {
      const b = this.state.activeBullets[i];
      b.velocity.y += b.gravity * delta * 2.0;
      b.mesh.position.addScaledVector(b.velocity, delta);

      // Raycast hit against scene colliders along velocity step
      const stepDist = b.velocity.length() * delta;
      const ray = new THREE.Raycaster(b.position, b.velocity.clone().normalize(), 0, stepDist);
      const hits = ray.intersectObjects(this.scene.children, true);

      if (hits.length > 0 || (now - b.spawnTime) / 1000 > b.lifetime || b.mesh.position.y < -10) {
        if (hits.length > 0) {
          this.spawnBulletImpact(hits[0].point, hits[0].face ? hits[0].face.normal : new THREE.Vector3(0,1,0));
        }
        if (b.mesh.parent) b.mesh.parent.remove(b.mesh);
        this.state.activeBullets.splice(i, 1);
      }
    }
  }
};

window.NyxFPSKit = NyxFPSKit;
console.log('✔ [Nyx Engine] FPS / Shooter Genre Kit Loaded.');
