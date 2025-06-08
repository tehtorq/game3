# Game Enhancement TODO List

## 1. Objective-Based Gameplay
- [ ] **Energy Crystals System**
  - [ ] Create crystal entity with visual representation
  - [ ] Spawn crystals at strategic terrain locations
  - [ ] Collection mechanics and particle effects
  - [ ] Crystal counter in HUD
  - [ ] Victory condition when all collected

- [x] **Enemy Bases**
  - [x] Design base structure (multi-part destructible)
  - [x] Place bases at terrain landmarks
  - [x] Base defenses (turrets, shield generators)
  - [x] Destruction sequence and effects
  - [x] Stop enemy spawning when base destroyed

- [ ] **Rescue Missions**
  - [ ] Friendly ship AI and pathfinding
  - [ ] Escort mechanics
  - [ ] Distress beacon system
  - [ ] Success/failure conditions

- [ ] **Territory Control**
  - [ ] Zone capture mechanics
  - [ ] Visual indicators for controlled areas
  - [ ] Resource generation from zones
  - [ ] Enemy counter-attacks

- [ ] **Scanner System**
  - [ ] Long-range detection mechanics
  - [ ] Radar/compass UI element
  - [ ] Different signatures for different objectives
  - [ ] Scanner upgrade system

## 2. Better Flight Physics
- [x] **Momentum & Inertia**
  - [x] Velocity-based movement instead of direct control
  - [x] Separate thrust and rotation controls
  - [x] Drift mechanics
  - [x] Max speed limits

- [x] **Enhanced Banking**
  - [x] Automatic banking based on turn rate
  - [x] Visual ship tilt
  - [x] Affects turn radius

- [x] **Altitude Effects**
  - [x] Thinner air at high altitude (less drag)
  - [x] Ground effect near terrain
  - [ ] Turbulence in certain areas

- [x] **Air Brakes**
  - [x] Quick deceleration key
  - [x] Visual brake indicators
  - [ ] Heat buildup from braking

- [x] **Afterburner System**
  - [x] Limited fuel gauge
  - [x] Regeneration over time
  - [ ] Visual thrust effects
  - [ ] Heat management

- [x] **Gravity System**
  - [x] Constant downward force
  - [ ] Stronger near massive terrain
  - [x] Affects projectiles

## 3. Enemy Spawn Improvements
- [x] **Enemy Patrols**
  - [x] Waypoint system for enemies
  - [x] Patrol routes around terrain
  - [ ] Alert states (patrol/search/attack)

- [x] **Enemy Nests/Bases**
  - [x] Fixed spawn locations
  - [x] Destructible spawners
  - [x] Increasing difficulty near bases
  - [x] Visual spawn animations

- [x] **Alert System**
  - [x] Detection ranges
  - [x] Alert propagation to nearby enemies
  - [x] Stealth mechanics
  - [x] Alert level indicators

- [ ] **Ambush Points**
  - [ ] Enemies hidden until player approaches
  - [ ] Terrain-based hiding spots
  - [ ] Surprise attack bonuses

- [ ] **Enemy Migration**
  - [ ] Groups move between bases
  - [ ] Supply convoys
  - [ ] Interceptable formations

## 4. Terrain Interaction
- [ ] **Canyon Flying**
  - [ ] Narrow passage generation
  - [ ] Collision detection improvements
  - [ ] Speed bonuses in canyons
  - [ ] Enemy-free zones

- [ ] **Caves/Tunnels**
  - [ ] Underground areas
  - [ ] Hidden entrances
  - [ ] Treasure rooms
  - [ ] Unique enemies

- [ ] **Destructible Terrain**
  - [ ] Certain rocks/structures can be destroyed
  - [ ] Chain reactions
  - [ ] Strategic destruction
  - [ ] Debris physics

- [ ] **Weather Effects**
  - [ ] Fog system with visibility reduction
  - [ ] Wind forces at altitude
  - [ ] Lightning storms
  - [ ] Weather transitions

- [ ] **Landmarks**
  - [ ] Unique terrain features
  - [ ] Named locations
  - [ ] Navigation aids
  - [ ] Strategic importance

## 5. Progression System
- [ ] **Ship Upgrades**
  - [ ] Collectible upgrade parts
  - [ ] Upgrade menu
  - [ ] Speed/armor/weapon improvements
  - [ ] Visual changes

- [ ] **Multiple Weapon Types**
  - [ ] Laser (current)
  - [ ] Homing missiles
  - [ ] Bombs for ground targets
  - [ ] EMP for shields
  - [ ] Weapon switching UI

- [x] **Shield System**
  - [x] Regenerating shields
  - [x] Shield strength indicator
  - [ ] Directional shields
  - [x] Shield overload effects

- [ ] **Checkpoint System**
  - [ ] Save points on map
  - [ ] Progress persistence
  - [ ] Respawn mechanics
  - [ ] Checkpoint activation effects

- [ ] **Mission Structure**
  - [ ] Mission briefings
  - [ ] Multi-objective missions
  - [ ] Mission completion rewards
  - [ ] Unlock system

## 6. HUD Improvements
- [x] **Minimap**
  - [x] Terrain elevation display
  - [x] Enemy positions
  - [x] Objective markers
  - [x] Zoom levels

- [x] **Altitude Indicator**
  - [x] Height above terrain
  - [ ] Altitude warnings
  - [ ] Optimal altitude indicators

- [x] **Speed/Momentum Display**
  - [x] Velocity vector indicator
  - [x] Speed gauge
  - [ ] G-force indicator
  - [ ] Stall warnings

- [x] **Objective Markers**
  - [ ] 3D markers in world
  - [ ] Distance indicators
  - [x] Off-screen indicators
  - [ ] Priority system

- [ ] **Threat Indicator**
  - [ ] Incoming missile warnings
  - [ ] Damage direction indicator
  - [ ] Enemy lock-on warnings
  - [ ] Proximity alerts

## Implementation Order (Suggested)
1. **Better Flight Physics** - Foundation for everything else
2. **HUD Improvements** - Better feedback for player
3. **Enemy Spawn Improvements** - More strategic gameplay
4. **Objective-Based Gameplay** - Give player goals
5. **Terrain Interaction** - Make world more interesting
6. **Progression System** - Long-term engagement

## Technical Considerations
- [ ] Performance optimization for larger world
- [ ] Save/load system
- [ ] Settings menu
- [ ] Sound effects integration
- [ ] Particle system improvements
- [ ] LOD system for distant objects