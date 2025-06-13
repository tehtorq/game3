# Enemy Types and Behaviors

## Basic Enemies

### Cube (Green)
- **Movement**: Hover - maintains altitude above terrain
- **Behavior**: Basic patrol with wandering movement
- **Speed**: 200-300 units/sec
- **Detection**: 1500 unit range, 126° FOV
- **Special**: None
- **Threat Level**: Low

### Pyramid (Red) 
- **Movement**: Diving - swooping attack patterns
- **Behavior**: Aggressive dive attacks when player detected
- **Speed**: 350-450 units/sec (fastest basic enemy)
- **Detection**: 2500 unit range, 108° FOV
- **Special**: Can shoot projectiles
- **Threat Level**: Medium

### Spinner (Orange)
- **Movement**: Orbital - circles around spawn point
- **Behavior**: Complex spinning patterns with multiple rotating parts
- **Speed**: 250-350 units/sec
- **Detection**: 2000 unit range, 144° FOV
- **Special**: Visual complexity can be disorienting
- **Threat Level**: Low-Medium

## Advanced Enemies

### Hunter (Magenta)
- **Movement**: Tracking - actively pursues player
- **Behavior**: Predicts player movement, leads shots, highly aggressive
- **Speed**: 300-400 units/sec
- **Detection**: 4000 unit range (longest), 90° FOV
- **Special**: Can shoot, excellent at alerting other enemies
- **Threat Level**: High

### Guardian (Gray)
- **Movement**: Patrol - follows waypoints around bases
- **Behavior**: Defensive, guards specific areas, 6-waypoint patrol routes
- **Speed**: 150-250 units/sec (slower but tanky)
- **Detection**: 1800 unit range, 162° FOV
- **Special**: Can shoot, higher health
- **Threat Level**: Medium

### Laser (Blue)
- **Movement**: Hover with slow approach
- **Behavior**: Sweeping laser attacks that track player
- **Speed**: 100-150 units/sec (very slow)
- **Detection**: 3000 unit range, 72° FOV
- **Special**: Continuous laser beam attack (3 second duration)
- **Threat Level**: High (area denial)

## Swarm Enemies

### Swarm (Yellow)
- **Movement**: Flocking - fast erratic movement in groups
- **Behavior**: Attacks in coordinated swarms, very aggressive
- **Speed**: 400-500 units/sec (fastest enemy)
- **Detection**: 1200 unit range, 180° FOV
- **Special**: Excellent communication (3000 unit alert range)
- **Threat Level**: Low individually, High in groups

### Carrier (Dark Gray)
- **Movement**: Drifting - slow movement at high altitude
- **Behavior**: Spawns 3 Swarm enemies every 4 seconds
- **Speed**: 60-100 units/sec (slowest)
- **Detection**: 3000 unit range, 162° FOV
- **Special**: Continuously spawns Swarm enemies
- **Threat Level**: High (force multiplier)

## Special Ability Enemies

### Phaser (Purple)
- **Movement**: Teleport - warps to new positions
- **Behavior**: Sniper that teleports every 2 seconds, fires after teleport
- **Speed**: 0 (teleports instead)
- **Detection**: 5000 unit range (sniper), 72° FOV
- **Special**: Teleportation, precise shots
- **Threat Level**: High

### Shield (Cyan)
- **Movement**: Stationary
- **Behavior**: Generates protective shields for nearby enemies
- **Speed**: 80-120 units/sec
- **Detection**: 2000 unit range, 270° FOV
- **Special**: 200 unit radius shield bubble protecting allies
- **Threat Level**: Medium (support)

### Bomber (Dark Red)
- **Movement**: Drifting at medium altitude
- **Behavior**: Drops explosive mines (5 total)
- **Speed**: 120-180 units/sec
- **Detection**: 2200 unit range, 126° FOV
- **Special**: Mine deployment
- **Threat Level**: Medium

### Disruptor (Electric Green)
- **Movement**: Hover
- **Behavior**: Charges and releases slowing waves
- **Speed**: 150-200 units/sec
- **Detection**: 2500 unit range, 144° FOV
- **Special**: Slowing wave (3 second effect on player)
- **Threat Level**: Medium (debuff)

### Reflector (Silver)
- **Movement**: Orbital, always faces player
- **Behavior**: Reflects enemy bullets back at player
- **Speed**: 180-250 units/sec
- **Detection**: 2000 unit range, 108° FOV
- **Special**: Bullet reflection
- **Threat Level**: Medium-High

### Vortex (Deep Purple)
- **Movement**: Stationary
- **Behavior**: Creates gravity well pulling player
- **Speed**: 0 (stationary)
- **Detection**: 1500 unit range, 360° FOV
- **Special**: 300 unit pull strength, affects player movement
- **Threat Level**: High (area control)

## Alert System

All enemies use a 5-state alert system:

1. **Unaware**: Normal patrol behavior
2. **Suspicious**: Heard something, investigating
3. **Alert**: Spotted player, actively attacking
4. **Searching**: Lost player, searching last known position
5. **Returning**: Giving up search, returning to patrol

### Detection Methods:
- **Visual**: Based on FOV and range
- **Audio**: Hearing range (800-2000 units)
- **Communication**: Alert enemies notify nearby allies

### Alert Durations:
- Alert state: 10 seconds (refreshed on sight)
- Searching: 30 seconds before giving up
- Communication range: 1200-3000 units

## Base Spawning

Enemies spawn from bases with different configurations:

- **Small Base**: 5-8 enemies per spawn
- **Medium Base**: More variety, faster spawn rate
- **Large Base**: Heavy enemies, turret support
- **Fortress**: All enemy types, maximum spawn rate

## Combat Tips

1. **Priority Targets**: Carriers, Shield generators, Vortex creators
2. **Dangerous Combinations**: 
   - Shield + any attacker (invulnerable)
   - Vortex + Laser (movement restriction + damage)
   - Disruptor + Hunter (slow + pursuit)
3. **Weak Points**: 
   - Most enemies vulnerable during attack animations
   - Teleporting Phasers vulnerable right after teleport
   - Carriers defenseless themselves