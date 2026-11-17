// =============================================================================
// NYX ENGINE — OPEN WORLD & AI SCALE KIT (TIME, WEATHER, PERCEPTION, AI STATE)
// =============================================================================

const NyxOpenWorldAIKit = {
  // 1. DYNAMIC TIME & WEATHER CONTROLLER
  environment: {
    timeOfDay: 14.0, // 0.0 to 24.0 hours (14:00 = 2 PM)
    timeScale: 60.0, // 1 real sec = 60 game secs (1 game day = 24 real minutes)
    weather: 'clear', // clear, overcast, rain, fog, snow
    weatherIntensity: 0.0, // 0.0 to 1.0 transition
    sunLight: null,
    ambientLight: null,
    skyColor: new THREE.Color(),
    groundWetness: 0.0
  },

  // 2. AI AGENTS REGISTRY
  aiAgents: [],

  // Initialize Environment with Scene Lights
  init(scene, sunLight, ambientLight) {
    this.scene = scene;
    this.environment.sunLight = sunLight;
    this.environment.ambientLight = ambientLight;
    this.updateSunOrbit();
    console.log('✔ [OpenWorldAIKit] Open World & AI Scale Kit initialized.');
  },

  // Set Specific Time of Day (00:00 to 24:00)
  setTimeOfDay(hours) {
    this.environment.timeOfDay = Math.max(0, Math.min(24, hours));
    this.updateSunOrbit();
  },

  // Set Specific Weather
  setWeather(weatherType) {
    this.environment.weather = weatherType;
    console.log(`[Weather] Transitioning weather to: ${weatherType}`);
  },

  // Orbiting Sun & Dynamic Sky Lighting
  updateSunOrbit() {
    const hours = this.environment.timeOfDay;
    // Angle: 6:00 = dawn, 12:00 = zenith, 18:00 = dusk, 0:00 = midnight
    const angle = ((hours - 6) / 24) * Math.PI * 2;
    const sunDist = 50.0;

    const sunX = Math.cos(angle) * sunDist;
    const sunY = Math.sin(angle) * sunDist;
    const sunZ = 20.0;

    if (this.environment.sunLight) {
      this.environment.sunLight.position.set(sunX, Math.max(-5, sunY), sunZ);
      
      // Calculate Sunlight Intensity & Sky Colors based on Sun Elevation
      const isDay = sunY > 0;
      if (isDay) {
        const elevation = sunY / sunDist; // 0 to 1
        this.environment.sunLight.intensity = Math.max(0.1, elevation * 1.4);
        
        if (elevation < 0.25) {
          // Dawn / Dusk Golden Hour
          this.environment.sunLight.color.setHex(0xffaa55);
          if (this.scene) this.scene.background = new THREE.Color(0x3a1e28);
        } else {
          // Noon Bright Daylight
          this.environment.sunLight.color.setHex(0xffffff);
          if (this.scene) this.scene.background = new THREE.Color(0x181c24);
        }
      } else {
        // Night Time (Moonlight)
        this.environment.sunLight.intensity = 0.08;
        this.environment.sunLight.color.setHex(0x334466);
        if (this.scene) this.scene.background = new THREE.Color(0x050810);
      }
    }
  },

  // 3. AI AGENT FACTORY (Sight Cone, Hearing, Behavior Tree)
  createAIAgent(name = 'Enemy_Grunt', position = new THREE.Vector3(0, 1, 0)) {
    const group = new THREE.Group();
    group.name = name;
    group.position.copy(position);

    // Visual Mesh: Enemy Body
    const bodyGeo = new THREE.CylinderGeometry(0.35, 0.35, 1.7, 8);
    const bodyMat = new THREE.MeshStandardMaterial({ color: 0x991b1b, roughness: 0.5 }); // Dark Red
    const bodyMesh = new THREE.Mesh(bodyGeo, bodyMat);
    bodyMesh.position.y = 0.85;
    bodyMesh.castShadow = true;
    group.add(bodyMesh);

    // Head / Eye direction indicator
    const headGeo = new THREE.SphereGeometry(0.25, 8, 8);
    const headMat = new THREE.MeshStandardMaterial({ color: 0xef4444 });
    const headMesh = new THREE.Mesh(headGeo, headMat);
    headMesh.position.set(0, 1.85, 0.15);
    group.add(headMesh);

    const agentData = {
      name: name,
      isEnemy: true,
      hasCollision: true,
      health: 100,
      maxHealth: 100,
      state: 'patrol', // patrol, investigate, chase, attack, flee
      patrolPoints: [
        position.clone(),
        position.clone().add(new THREE.Vector3(6, 0, 0)),
        position.clone().add(new THREE.Vector3(6, 0, 6)),
        position.clone().add(new THREE.Vector3(0, 0, 6))
      ],
      currentPatrolIdx: 0,
      moveSpeed: 2.2,
      sightRange: 16.0,
      sightAngle: Math.PI / 2.2, // 82 deg cone
      hearingRadius: 18.0,
      lastKnownPlayerPos: null,
      attackCooldown: 1.2,
      lastAttackTime: 0
    };

    group.userData = agentData;
    this.aiAgents.push(group);

    if (this.scene) this.scene.add(group);
    console.log(`[AI] Spawned AI Agent "${name}" at (${position.x}, ${position.y}, ${position.z})`);
    return group;
  },

  // Sound Broadcast Event (Gunshots, Footsteps)
  broadcastSound(originPos, noiseRadius = 20.0, noiseType = 'gunshot') {
    this.aiAgents.forEach(agent => {
      const dist = agent.position.distanceTo(originPos);
      if (dist <= noiseRadius) {
        agent.userData.lastKnownPlayerPos = originPos.clone();
        if (agent.userData.state !== 'chase' && agent.userData.state !== 'attack') {
          agent.userData.state = 'investigate';
          console.log(`[AI Hearing] "${agent.name}" heard ${noiseType} (${dist.toFixed(1)}m away). Investigating...`);
        }
      }
    });
  },

  // Per-Frame AI Behavior Tree & Sensory Perception Loop
  update(delta, playerPos) {
    // 1. Advance Time
    this.environment.timeOfDay = (this.environment.timeOfDay + (delta * this.environment.timeScale / 3600)) % 24;
    this.updateSunOrbit();

    // 2. Weather Surface Wetness
    if (this.environment.weather === 'rain') {
      this.environment.groundWetness = Math.min(1.0, this.environment.groundWetness + delta * 0.1);
    } else {
      this.environment.groundWetness = Math.max(0.0, this.environment.groundWetness - delta * 0.05);
    }

    // 3. Update AI Agents
    if (!playerPos) return;

    this.aiAgents.forEach(agent => {
      const d = agent.userData;
      if (!agent.visible || d.health <= 0) return;

      const toPlayer = playerPos.clone().sub(agent.position);
      const distToPlayer = toPlayer.length();

      // Sensory Perception: Sight Cone
      const facing = new THREE.Vector3(0, 0, 1).applyQuaternion(agent.quaternion);
      const angleToPlayer = facing.angleTo(toPlayer.clone().normalize());

      const canSeePlayer = (distToPlayer <= d.sightRange && angleToPlayer <= d.sightAngle / 2);

      if (canSeePlayer) {
        d.lastKnownPlayerPos = playerPos.clone();
        if (d.health < 25) {
          d.state = 'flee';
        } else if (distToPlayer <= 2.2) {
          d.state = 'attack';
        } else {
          d.state = 'chase';
        }
      }

      // Execute State Machine
      if (d.state === 'patrol') {
        const target = d.patrolPoints[d.currentPatrolIdx];
        const toTarget = target.clone().sub(agent.position);
        toTarget.y = 0;

        if (toTarget.length() < 0.6) {
          d.currentPatrolIdx = (d.currentPatrolIdx + 1) % d.patrolPoints.length;
        } else {
          toTarget.normalize();
          agent.position.addScaledVector(toTarget, d.moveSpeed * delta);
          agent.lookAt(agent.position.clone().add(toTarget));
        }

      } else if (d.state === 'investigate') {
        if (d.lastKnownPlayerPos) {
          const toLKP = d.lastKnownPlayerPos.clone().sub(agent.position);
          toLKP.y = 0;
          if (toLKP.length() < 1.0) {
            d.state = 'patrol'; // reached sound origin, resumed patrol
            d.lastKnownPlayerPos = null;
          } else {
            toLKP.normalize();
            agent.position.addScaledVector(toLKP, d.moveSpeed * delta);
            agent.lookAt(agent.position.clone().add(toLKP));
          }
        }

      } else if (d.state === 'chase') {
        toPlayer.y = 0;
        toPlayer.normalize();
        agent.position.addScaledVector(toPlayer, d.moveSpeed * 1.5 * delta);
        agent.lookAt(agent.position.clone().add(toPlayer));

        if (distToPlayer <= 2.0) d.state = 'attack';

      } else if (d.state === 'attack') {
        agent.lookAt(playerPos.x, agent.position.y, playerPos.z);
        const now = performance.now();
        if (now - d.lastAttackTime >= d.attackCooldown * 1000) {
          d.lastAttackTime = now;
          console.log(`⚔️ [AI Attack] "${agent.name}" attacked Player!`);
        }
        if (distToPlayer > 2.5) d.state = 'chase';

      } else if (d.state === 'flee') {
        // Run opposite of player
        const fleeDir = agent.position.clone().sub(playerPos);
        fleeDir.y = 0;
        fleeDir.normalize();
        agent.position.addScaledVector(fleeDir, d.moveSpeed * 1.8 * delta);
        agent.lookAt(agent.position.clone().add(fleeDir));
      }
    });
  }
};

window.NyxOpenWorldAIKit = NyxOpenWorldAIKit;
console.log('✔ [Nyx Engine] Open World & AI Scale Kit Loaded.');
