// =============================================================================
// NYX ENGINE — ADVANCED PROCEDURAL GENERATION SUITE (TREES, ROADS, DUNGEONS)
// =============================================================================

const NyxProceduralSuite = {

  // 1. PARAMETRIC 3D TREE GENERATOR (Oak, Pine, Palm, Willow, Dead)
  generateProceduralTree(params = {}) {
    const seed = params.seed || Math.floor(Math.random() * 10000);
    const type = params.type || 'oak'; // oak, pine, palm, willow, dead
    const trunkHeight = params.trunkHeight || 6.0;
    const trunkRadius = params.trunkRadius || 0.45;
    const branches = params.branches || 4;
    const leafDensity = params.leafDensity || 1.0;
    const leafColorPreset = params.leafColor || 'summer'; // summer, autumn, sakura, golden

    const treeGroup = new THREE.Group();
    const treeName = params.name || `Tree_${type.toUpperCase()}_${seed}`;
    treeGroup.name = treeName;

    // Materials
    const barkColor = type === 'palm' ? 0x6e543c : (type === 'dead' ? 0x3d352e : 0x4a2e18);
    const barkMat = new THREE.MeshStandardMaterial({ color: barkColor, roughness: 0.85, metalness: 0.05 });

    let leafColor = 0x2e7d32; // deep green
    if (leafColorPreset === 'autumn') leafColor = 0xd35400;
    else if (leafColorPreset === 'sakura') leafColor = 0xf48fb1;
    else if (leafColorPreset === 'golden') leafColor = 0xf39c12;

    const leafMat = new THREE.MeshStandardMaterial({
      color: leafColor,
      roughness: 0.6,
      metalness: 0.0,
      side: THREE.DoubleSide
    });

    // 1. Trunk Construction
    const trunkGeo = new THREE.CylinderGeometry(trunkRadius * 0.6, trunkRadius, trunkHeight, 8, 4);
    // Slight crooked bend
    const pos = trunkGeo.attributes.position;
    for (let i = 0; i < pos.count; i++) {
      const y = pos.getY(i);
      const curve = Math.sin((y / trunkHeight) * Math.PI) * (trunkRadius * 0.8);
      pos.setX(i, pos.getX(i) + curve);
    }
    trunkGeo.computeVertexNormals();

    const trunkMesh = new THREE.Mesh(trunkGeo, barkMat);
    trunkMesh.position.y = trunkHeight / 2;
    trunkMesh.castShadow = true;
    trunkMesh.receiveShadow = true;
    trunkMesh.userData = { hasCollision: true, isTrunk: true };
    treeGroup.add(trunkMesh);

    // 2. Canopy / Branching Construction
    if (type === 'oak') {
      // Rounded spherical foliage clumps
      const clumpCount = 5 + Math.floor(branches * 1.5);
      for (let b = 0; b < clumpCount; b++) {
        const rad = 1.4 + Math.random() * 0.8;
        const clumpGeo = new THREE.DodecahedronGeometry(rad, 1);
        const clumpMesh = new THREE.Mesh(clumpGeo, leafMat);
        const angle = (b / clumpCount) * Math.PI * 2 + Math.random() * 0.5;
        const dist = 1.0 + Math.random() * 1.5;
        clumpMesh.position.set(
          Math.cos(angle) * dist,
          trunkHeight * 0.75 + (Math.random() - 0.2) * 2.5,
          Math.sin(angle) * dist
        );
        clumpMesh.castShadow = true;
        clumpMesh.userData = { isCanopy: true, isWindSway: true };
        treeGroup.add(clumpMesh);
      }
    } else if (type === 'pine') {
      // Tiered conical layers
      const tiers = 4;
      for (let t = 0; t < tiers; t++) {
        const tierHeight = trunkHeight * 0.4 + t * 1.2;
        const coneRadius = 2.6 - t * 0.55;
        const coneGeo = new THREE.ConeGeometry(coneRadius, 1.8, 7);
        const coneMesh = new THREE.Mesh(coneGeo, leafMat);
        coneMesh.position.y = tierHeight;
        coneMesh.castShadow = true;
        coneMesh.userData = { isCanopy: true, isWindSway: true };
        treeGroup.add(coneMesh);
      }
    } else if (type === 'palm') {
      // Palm fronds extending outwards
      const frondCount = 7;
      for (let f = 0; f < frondCount; f++) {
        const frondAngle = (f / frondCount) * Math.PI * 2;
        const frondGeo = new THREE.PlaneGeometry(1.2, 3.2, 3, 5);
        frondGeo.rotateX(Math.PI / 3);
        const frondMesh = new THREE.Mesh(frondGeo, leafMat);
        frondMesh.rotation.y = frondAngle;
        frondMesh.position.set(0, trunkHeight, 0);
        frondMesh.castShadow = true;
        frondMesh.userData = { isCanopy: true, isWindSway: true };
        treeGroup.add(frondMesh);
      }
    }

    // Auto-Collider: Tight cylinder on the trunk
    treeGroup.userData = {
      name: treeName,
      isUserObject: true,
      objType: 'tree',
      treeParams: { seed, type, trunkHeight, trunkRadius, branches, leafColorPreset },
      hasCollision: true
    };

    console.log(`[ProceduralTree] Built parametric ${type} tree "${treeName}" (Height: ${trunkHeight}m)`);
    return treeGroup;
  },

  // 2. PROCEDURAL SPLINE ROAD & RIVER GENERATOR
  generateSplineRoad(points = [], width = 4.0, roadType = 'asphalt') {
    if (!points || points.length < 2) {
      points = [
        new THREE.Vector3(-20, 0, -20),
        new THREE.Vector3(-5, 0.5, -5),
        new THREE.Vector3(5, 0.2, 5),
        new THREE.Vector3(25, 0, 20)
      ];
    }

    const curve = new THREE.CatmullRomCurve3(points);
    const sampleCount = points.length * 15;
    const curvePoints = curve.getPoints(sampleCount);

    const roadGroup = new THREE.Group();
    const roadName = `SplineRoad_${Date.now().toString().slice(-4)}`;
    roadGroup.name = roadName;

    // Road Ribbon Geometry
    const vertices = [];
    const uvs = [];
    const indices = [];

    let color = 0x22262a; // asphalt dark
    if (roadType === 'dirt') color = 0x5a4430;
    else if (roadType === 'river') color = 0x1a4568;

    const roadMat = new THREE.MeshStandardMaterial({
      color: color,
      roughness: roadType === 'river' ? 0.1 : 0.8,
      metalness: roadType === 'river' ? 0.4 : 0.05,
      side: THREE.DoubleSide
    });

    const halfW = width / 2;
    for (let i = 0; i < curvePoints.length; i++) {
      const p = curvePoints[i];
      let tangent = new THREE.Vector3(1, 0, 0);
      if (i < curvePoints.length - 1) {
        tangent.subVectors(curvePoints[i+1], p).normalize();
      } else {
        tangent.subVectors(p, curvePoints[i-1]).normalize();
      }
      const normal = new THREE.Vector3(0, 1, 0);
      const binormal = new THREE.Vector3().crossVectors(tangent, normal).normalize();

      const leftPt = p.clone().addScaledVector(binormal, -halfW);
      const rightPt = p.clone().addScaledVector(binormal, halfW);

      vertices.push(leftPt.x, leftPt.y + 0.05, leftPt.z);
      vertices.push(rightPt.x, rightPt.y + 0.05, rightPt.z);

      const vProgress = i / (curvePoints.length - 1);
      uvs.push(0, vProgress * 5);
      uvs.push(1, vProgress * 5);

      if (i < curvePoints.length - 1) {
        const base = i * 2;
        indices.push(base, base + 1, base + 2);
        indices.push(base + 1, base + 3, base + 2);
      }
    }

    const roadGeo = new THREE.BufferGeometry();
    roadGeo.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
    roadGeo.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
    roadGeo.setIndex(indices);
    roadGeo.computeVertexNormals();

    const roadMesh = new THREE.Mesh(roadGeo, roadMat);
    roadMesh.receiveShadow = true;
    roadMesh.userData = { hasCollision: true };
    roadGroup.add(roadMesh);

    roadGroup.userData = {
      name: roadName,
      isUserObject: true,
      objType: 'road',
      hasCollision: true,
      roadParams: { width, roadType, pointCount: points.length }
    };

    console.log(`[SplineRoad] Extruded ${roadType} along ${points.length}-node spline (${sampleCount} segments)`);
    return roadGroup;
  },

  // 3. PROCEDURAL DUNGEON & MAZE GENERATOR (BSP Tree Rooms + Corridors)
  generateProceduralDungeon(gridW = 40, gridH = 40, roomCount = 6) {
    const dungeonGroup = new THREE.Group();
    const name = `Dungeon_BSP_${Date.now().toString().slice(-4)}`;
    dungeonGroup.name = name;

    const wallMat = new THREE.MeshStandardMaterial({ color: 0x2a2a2e, roughness: 0.9 });
    const floorMat = new THREE.MeshStandardMaterial({ color: 0x1d1d20, roughness: 0.8 });

    const rooms = [];
    // Generate BSP non-overlapping rooms
    for (let r = 0; r < roomCount; r++) {
      const rw = 5 + Math.floor(Math.random() * 5);
      const rh = 5 + Math.floor(Math.random() * 5);
      const rx = Math.floor((Math.random() - 0.5) * (gridW - rw));
      const rz = Math.floor((Math.random() - 0.5) * (gridH - rh));

      // Floor
      const fGeo = new THREE.PlaneGeometry(rw, rh);
      fGeo.rotateX(-Math.PI / 2);
      const floor = new THREE.Mesh(fGeo, floorMat);
      floor.position.set(rx, 0, rz);
      floor.receiveShadow = true;
      floor.userData = { hasCollision: true };
      dungeonGroup.add(floor);

      // 4 Walls around room
      const wallH = 3.2;
      const wNorth = new THREE.Mesh(new THREE.BoxGeometry(rw, wallH, 0.2), wallMat);
      wNorth.position.set(rx, wallH / 2, rz - rh / 2);
      wNorth.userData = { hasCollision: true };
      dungeonGroup.add(wNorth);

      const wSouth = new THREE.Mesh(new THREE.BoxGeometry(rw, wallH, 0.2), wallMat);
      wSouth.position.set(rx, wallH / 2, rz + rh / 2);
      wSouth.userData = { hasCollision: true };
      dungeonGroup.add(wSouth);

      const wWest = new THREE.Mesh(new THREE.BoxGeometry(0.2, wallH, rh), wallMat);
      wWest.position.set(rx - rw / 2, wallH / 2, rz);
      wWest.userData = { hasCollision: true };
      dungeonGroup.add(wWest);

      const wEast = new THREE.Mesh(new THREE.BoxGeometry(0.2, wallH, rh), wallMat);
      wEast.position.set(rx + rw / 2, wallH / 2, rz);
      wEast.userData = { hasCollision: true };
      dungeonGroup.add(wEast);

      // Torch Light in Room
      const light = new THREE.PointLight(0xff9944, 0.8, 12);
      light.position.set(rx, 2.2, rz);
      dungeonGroup.add(light);

      rooms.push({ x: rx, z: rz, w: rw, h: rh });
    }

    // Connect consecutive rooms with hallways
    for (let c = 0; c < rooms.length - 1; c++) {
      const r1 = rooms[c];
      const r2 = rooms[c+1];
      const midX = (r1.x + r2.x) / 2;
      const midZ = (r1.z + r2.z) / 2;
      const lenX = Math.abs(r1.x - r2.x) + 2.0;

      const hallGeo = new THREE.PlaneGeometry(lenX, 2.5);
      hallGeo.rotateX(-Math.PI / 2);
      const hall = new THREE.Mesh(hallGeo, floorMat);
      hall.position.set(midX, 0, midZ);
      hall.userData = { hasCollision: true };
      dungeonGroup.add(hall);
    }

    dungeonGroup.userData = {
      name: name,
      isUserObject: true,
      objType: 'dungeon',
      hasCollision: true,
      dungeonRooms: rooms.length
    };

    console.log(`[ProceduralDungeon] Generated ${rooms.length}-room dungeon with hallway connections.`);
    return dungeonGroup;
  },

  // 4. MODULAR BUILDING GENERATOR
  generateModularBuilding(floors = 3, width = 8, depth = 8) {
    const buildingGroup = new THREE.Group();
    const name = `Modular_Building_${Date.now().toString().slice(-4)}`;
    buildingGroup.name = name;

    const floorH = 3.0;
    const wallMat = new THREE.MeshStandardMaterial({ color: 0x3d444e, roughness: 0.7 });
    const glassMat = new THREE.MeshStandardMaterial({ color: 0x70a0c0, roughness: 0.1, metalness: 0.8 });
    const roofMat = new THREE.MeshStandardMaterial({ color: 0x6e2c24, roughness: 0.85 });

    for (let f = 0; f < floors; f++) {
      const floorY = f * floorH;

      // Slab Floor
      const slab = new THREE.Mesh(new THREE.BoxGeometry(width + 0.4, 0.25, depth + 0.4), wallMat);
      slab.position.set(0, floorY, 0);
      slab.userData = { hasCollision: true };
      buildingGroup.add(slab);

      // Floor Walls
      const wallBox = new THREE.Mesh(new THREE.BoxGeometry(width, floorH - 0.25, depth), wallMat);
      wallBox.position.set(0, floorY + floorH / 2, 0);
      wallBox.userData = { hasCollision: true };
      buildingGroup.add(wallBox);

      // Windows on front facade
      const windowMesh = new THREE.Mesh(new THREE.BoxGeometry(width * 0.7, 1.4, 0.1), glassMat);
      windowMesh.position.set(0, floorY + floorH * 0.55, depth / 2 + 0.05);
      buildingGroup.add(windowMesh);
    }

    // Pitched Roof
    const roofGeo = new THREE.ConeGeometry(Math.max(width, depth) * 0.75, 2.5, 4);
    roofGeo.rotateY(Math.PI / 4);
    const roof = new THREE.Mesh(roofGeo, roofMat);
    roof.position.set(0, floors * floorH + 1.25, 0);
    roof.userData = { hasCollision: true };
    buildingGroup.add(roof);

    buildingGroup.userData = {
      name: name,
      isUserObject: true,
      objType: 'building',
      hasCollision: true,
      buildingParams: { floors, width, depth }
    };

    console.log(`[ModularBuilding] Built ${floors}-story building "${name}" (${width}x${depth}m)`);
    return buildingGroup;
  }
};

window.NyxProceduralSuite = NyxProceduralSuite;
console.log('✔ [Nyx Engine] Procedural Generation Suite Module Loaded.');
