# Procedural animation types

Procedural animation generates motion mathematically at runtime rather than relying on hand-authored keyframes. Below is a guide to every major category — what each technique does, how it works, and where it's used.

---

## Physics simulation

### Rigid body physics
Simulates objects as perfectly stiff solids that cannot deform. Each body has mass, a center of gravity, linear velocity, angular velocity, and a collision shape. Forces (gravity, explosions, friction) update these each frame via Newton's laws. Broad-phase collision detection finds candidate pairs; narrow-phase resolves exact contacts and computes impulses. Used for crates, barrels, vehicles, and anything that bounces or tumbles. Most engines (PhysX, Havok, Chaos, Bullet) run this on a dedicated physics thread.

### Soft body / jelly simulation
Extends rigid body ideas to objects that can deform. The mesh is treated as a network of point masses connected by springs or constraints. Each frame, spring forces pull points toward their rest lengths, while external forces (gravity, collisions) push them around. Position-Based Dynamics (PBD) is a popular modern solver — instead of integrating forces directly, it projects positions to satisfy constraints iteratively. Used for jelly, flesh, rubber, foam, and any squash-and-stretch material.

### Cloth and fabric
A specialization of soft body simulation where the mesh is thin and primarily resists bending and stretching. Common solvers include Verlet integration (each vertex tracks its previous position, making velocity implicit) and PBD constraint projection. Constraints enforce maximum stretch along edges, bend resistance between adjacent triangles, and collision avoidance with the character's body. GPU compute shaders run cloth constraints in parallel, allowing high-resolution cloth at real-time frame rates. Used for capes, flags, curtains, and character clothing.

### Fluid simulation
Models liquids and gases as fields of velocity, pressure, and density governed by the Navier-Stokes equations. Three main approaches are used in games:

- **Grid-based (Eulerian):** The world is divided into a voxel grid; each cell stores velocity and density. Advection moves quantities through the grid; a pressure solve enforces incompressibility. Expensive but visually rich.
- **Smoothed Particle Hydrodynamics (SPH):** Fluid is represented as thousands of particles that push and attract each other based on density estimates. Easily parallelized on the GPU.
- **FLIP (Fluid-Implicit-Particle):** Hybrid approach — particles carry velocity, a grid handles pressure. Best of both worlds; used in many VFX tools.

### Smoke and fire
Uses the same grid-based fluid solver as liquid simulation, with additional fields for temperature and fuel. Hot cells rise via buoyancy forces; fuel burns and produces heat; temperature drives emission color from cool red through orange to hot white. Voxel grids are typically low resolution (32³ to 128³) for real-time use, with noise layered on top to add detail. Fire is rendered by ray-marching through the density volume in a shader.

### Ocean waves
Real-time ocean surfaces use the **Gerstner wave** model or **FFT (Fast Fourier Transform) synthesis**. In the FFT approach, a spectrum of wave amplitudes and directions (usually the Phillips spectrum, based on wind speed and direction) is generated in frequency space, then inverse-FFT'd each frame to produce a height map. The GPU runs the FFT in milliseconds, making a full ocean surface with realistic swell, chop, and whitecaps feasible at 60fps. Used in sailing games, naval simulations, and open-world environments.

---

## Character and skeletal animation

### Inverse kinematics (IK)
Forward kinematics animates a skeleton by rotating joints from root to tip — straightforward but inflexible. IK inverts this: given a desired end-effector position (a hand reaching for a door handle, a foot landing on a step), solve for the joint rotations that achieve it.

Common algorithms:
- **CCD (Cyclic Coordinate Descent):** Iterates along the chain from tip to root, rotating each joint to minimize end-effector error. Simple and fast, but can produce unnatural poses.
- **FABRIK (Forward and Backward Reaching IK):** Alternates forward and backward passes, repositioning joints along the bone direction. Converges quickly and produces more natural limb shapes.
- **Jacobian / gradient descent:** Formulates IK as a numerical optimization problem. More expensive but handles constraints (joint limits, multiple end effectors) robustly.

Used everywhere: foot planting, hand reaching, look-at for heads, weapon aiming, and climbing.

### Procedural locomotion
Rather than blending between pre-authored walk/run cycles, procedural locomotion generates step placement, stride length, and body sway mathematically from the character's velocity and the terrain. Key components:

- **Foot planting:** Raycasts from each foot down to the terrain; IK adjusts the leg chain to match the ground height and slope
- **Step triggering:** Decides when to lift a foot based on how far it has strayed from the desired position under the moving body
- **Body sway and lean:** Procedural offsets applied to the hips and spine based on speed and turning rate

Used in games where characters traverse highly varied terrain — *Spider-Man*, *Horizon*, *Death Stranding*.

### Ragdoll and physics-animation blending
A ragdoll replaces a character's animation with full rigid body simulation — every bone becomes a physics object, connected by joints with angular limits. Pure ragdolls look loose and lifeless; the art is blending between keyframed animation and ragdoll physics.

Techniques include:
- **Transition blending:** Snap to ragdoll on death, then fade back to animation for a "get up" sequence
- **Partial ragdoll:** Physics drives the lower body while animation drives the upper body (or vice versa)
- **Procedural hit reactions:** A physics impulse is applied to the hit bone; the solver propagates it through the skeleton while animation continues driving the rest of the body

Unreal's **Physical Animation Component** and Unity's **Ragdoll Utility** both implement variations of this blending.

---

## Geometry and terrain

### Terrain generation
Procedural terrain starts with layered noise — typically Perlin or Simplex noise at multiple frequencies (octaves) summed together to produce fractal-like height variation. Additional passes apply:

- **Hydraulic erosion:** Simulates rainwater flowing downhill, carrying and depositing sediment. Cuts valleys, smooths peaks, and deposits alluvial fans. Runs as a GPU compute shader over millions of iterations.
- **Thermal erosion:** Material slides downhill when slope exceeds an angle of repose, producing scree fields and talus slopes.
- **GPU tessellation:** At runtime, the terrain mesh is subdivided based on camera distance, with displacement maps providing local detail. DirectX 11+ tessellation shaders handle this transparently.

### Mesh deformation
Vertices of a mesh are displaced at runtime by CPU or GPU code. Common forms:

- **Blend shapes / morph targets:** Two or more mesh poses stored as delta arrays; interpolating between them produces facial expressions, muscle flexion, and damage states
- **Skinning with secondary motion:** Bones are driven by animation, but a secondary pass adds jiggle, delay, or overshoot to fleshy parts — cheeks, hair, tails — using spring-damper systems per bone
- **Shader-driven deformation:** The vertex shader displaces positions using noise, a sine wave, or a texture lookup — used for waving flags, breathing chest, shimmering heat haze on geometry

### Vegetation and foliage
Individual blades of grass, leaves, and branches are too numerous for individual simulation. Instead:

- **GPU instancing** draws thousands of plant instances in a single draw call, each with a unique transform
- **A vertex shader** displaces each instance using the world-space position as a seed into a sine wave or noise function, driven by a wind direction and strength uniform
- **LOD and culling** remove distant vegetation and switch to impostor billboards at range
- **Interaction** — footsteps or vehicles pressing grass flat — is commonly handled by writing to a render texture that the vertex shader samples as an additional displacement

Used in virtually every open-world game for ambient environmental life.

---

## Behavior and crowds

### Flocking / Boids
Craig Reynolds' 1986 Boids algorithm produces lifelike collective motion from three local rules applied to each agent:

1. **Separation:** Steer away from neighbors that are too close
2. **Alignment:** Steer toward the average heading of nearby neighbors
3. **Cohesion:** Steer toward the average position of nearby neighbors

Each agent only looks at neighbors within a perception radius — there is no global coordinator. Complex emergent behaviors (splitting around obstacles, reforming, swirling) arise naturally. On the GPU, each agent's neighbor search is parallelized using spatial hashing or a grid structure, enabling millions of simultaneous agents. Used for birds, fish schools, insects, and abstract particle effects.

### Crowd simulation
Crowds of hundreds or thousands of human agents require more sophisticated behavior than Boids:

- **Flow fields:** A vector field is precomputed over the navmesh, pointing each cell toward the goal. Agents follow the field cheaply at runtime — no per-agent pathfinding required
- **Velocity obstacles (RVO/ORCA):** Each agent computes the set of velocities that would cause a collision with neighbors, then picks a velocity outside that set. Produces smooth, collision-free local avoidance with no central coordination
- **GPU agents:** Simple agents (spectators, distant pedestrians) are simulated entirely on the GPU using compute shaders — position, heading, and a state machine, updated in parallel
- **LOD behavior:** Nearby agents run full animation and collision avoidance; distant agents animate with cheaper shared skeletons and simplified movement

Used in stadium games (*FIFA*, *NBA 2K*), battle simulations, and open-world pedestrian systems.

### Cellular automata
Each cell in a grid updates its state based on the states of its neighbors, according to a fixed rule table. The classic example is Conway's Game of Life, but game-relevant applications include:

- **Fire spread:** A burning cell probabilistically ignites adjacent flammable cells; burnt cells transition to ash
- **Falling sand / powder simulations:** Each particle checks the cell below it; if empty, it falls. Different materials have different densities and interaction rules (water flows around sand, oil floats on water)
- **Biological growth:** Coral, mold, crystal growth — any spreading pattern with local interaction
- **Cellular noise:** Worley noise (used for water caustics, biological textures) is computed by finding the nearest seed point per cell — conceptually a CA variant

The grid update is embarrassingly parallel and maps naturally to a compute shader ping-ponging between two textures.

---

## Shader-driven animation

### Reaction-diffusion systems
Two virtual chemicals, A and B, diffuse through a 2D grid and react with each other. The Gray-Scott model is the most common:

- A is produced everywhere (feed rate F), diffuses quickly, and is consumed when it meets B
- B diffuses more slowly, is produced when A and B meet, and decays at rate K

Different F and K values produce strikingly different patterns — spots, stripes, coral-like branching, labyrinthine mazes. The simulation runs as a ping-pong fragment shader, reading the previous frame's concentration texture and writing the next. Used for organic surface textures, alien skin patterns, and shader art.

### Vertex animation textures (VAT)
A physics or cloth simulation is baked offline into a texture: each row is a frame of animation, each column is a vertex, and the RGB values encode the XYZ displacement from rest pose. At runtime, the vertex shader samples this texture at `(vertex_id, time)` and applies the displacement — replaying a complex simulation for near-zero runtime cost.

Used for:
- Destruction sequences where the result always looks the same
- Cloth and flag animations on static objects
- Large-scale vegetation animation pre-baked from a wind simulation

Common in mobile and VR where physics CPU budget is scarce.

### SDF raymarching
Signed Distance Fields represent geometry as a mathematical function: `f(x,y,z)` returns the distance to the nearest surface, negative inside and positive outside. Instead of rasterizing triangles, a fragment shader marches a ray through space, stepping forward by the SDF value until it hits the surface (value ≈ 0).

Procedural animation is trivial — you animate the SDF function itself. Morphing between two shapes means lerping between two SDF functions. Repeating structures, twists, bends, and organic swellings are a few lines of math. Used for:
- Shader toy-style abstract animation
- Metaball / blob effects
- Procedural planet and cloud rendering
- UI elements that smoothly morph between states

---

## Growth and structural animation

### L-systems and growth animation
Lindenmayer systems are a string-rewriting grammar originally developed to model plant growth. A short axiom string is repeatedly expanded by production rules — `F` might expand to `FF+[+F-F-F]-[-F+F+F]` — and the resulting string is interpreted as drawing commands (forward, turn, branch). Animating an L-system means visualizing intermediate expansion states, producing convincing branch-by-branch growth. Used for trees, vines, coral, and fractal terrain features.

### Destruction and fracturing
See above in the physics section — Voronoi fracturing, constraint graphs, hierarchical clusters. Worthy of its own document.

### Neural / ML-based animation
Machine learning increasingly drives character animation:

- **Motion matching:** A large database of motion capture clips is searched each frame for the clip whose future trajectory best matches the character's current velocity and desired direction. No blending trees, no state machines — just a nearest-neighbor search, now fast enough to run in real time
- **Neural state machines:** A small neural network is trained on motion data to predict the next pose given current pose and controller input. Produces extremely natural transitions between arbitrary states
- **Physics-informed neural networks:** The network is trained to produce poses that satisfy physical constraints (no foot sliding, no interpenetration), removing the need for explicit IK post-processing
- **NPC behavior:** Reinforcement learning trains agents to navigate, fight, or cooperate in ways that are difficult to hand-author. The policy network runs cheaply at inference time

Used in *FIFA* (motion matching), research engines like *Learned Motion Matching*, and increasingly in AAA character systems.

---

## Quick reference

| Category | Technique | Primary computation | GPU-friendly? |
|---|---|---|---|
| Physics | Rigid body | Force/impulse integration | Partially |
| Physics | Soft body | Spring/PBD constraint solving | Yes |
| Physics | Cloth | PBD / Verlet integration | Yes |
| Physics | Fluid | Navier-Stokes solver | Yes |
| Physics | Smoke/fire | Grid advection + combustion | Yes |
| Physics | Ocean waves | FFT spectrum synthesis | Yes |
| Character | Inverse kinematics | Chain solver (FABRIK/CCD) | Partially |
| Character | Procedural locomotion | Raycast + IK + spring | CPU |
| Character | Ragdoll blending | Rigid body + pose lerp | Partially |
| Geometry | Terrain | Noise + erosion simulation | Yes |
| Geometry | Mesh deformation | Blend shapes / vertex shaders | Yes |
| Geometry | Vegetation | Instance + vertex shader wind | Yes |
| Behavior | Flocking / Boids | Per-agent neighbor query | Yes |
| Behavior | Crowd simulation | Flow fields + RVO avoidance | Partially |
| Behavior | Cellular automata | Grid neighbor rules | Yes |
| Shader | Reaction-diffusion | Ping-pong fragment shader | Yes |
| Shader | Vertex animation textures | Texture lookup in vertex shader | Yes |
| Shader | SDF raymarching | Iterative ray stepping | Yes |
| Growth | L-systems | String rewriting + drawing | CPU |
| ML | Motion matching | Nearest-neighbor search | Partially |
| ML | Neural animation | Inference on small network | Yes |
