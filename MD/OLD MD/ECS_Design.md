Flecs is a fast and lightweight Entity Component System that lets you build games and simulations with millions of entities (join the Discord!). Here are some of the framework's highlights:

Fast and portable zero dependency C99 API
Modern type-safe C++17 API that doesn't use STL containers
First open source ECS with full support for Entity Relationships!
Fast native support for hierarchies and prefabs
Code base that builds in less than 5 seconds
Runs in the browser without modifications with emscripten
Cache friendly archetype/SoA storage that can process millions of entities every frame
Automatic component registration that works out of the box across shared libraries/DLLs
Write free functions with queries or run code automatically in systems
Run games on multiple CPU cores with a fast lockless scheduler
Verified on all major compilers and platforms with CI running more than 10000 tests
Integrated reflection framework with JSON serializer and support for runtime components
Unit annotations for components
Powerful query language with support for joins and inheritance
Statistics addon for profiling ECS performance
A web-based UI for monitoring & controlling your apps:
Flecs Explorer

To support the project, give it a star 🌟 !

What is an Entity Component System?
ECS is a way of organizing code and data that lets you build games that are larger, more complex and are easier to extend. Something is called an ECS when it:

Has entities that uniquely identify objects in a game
Has components which are datatypes that can be added to entities
Has systems which are functions that run for all entities matching a component query
For more information, check the ECS FAQ!

Show me the code!
C99 example:

typedef struct {
  float x, y;
} Position, Velocity;

void Move(ecs_iter_t *it) {
  Position *p = ecs_field(it, Position, 0);
  Velocity *v = ecs_field(it, Velocity, 1);

  for (int i = 0; i < it->count; i++) {
    p[i].x += v[i].x;
    p[i].y += v[i].y;
  }
}

int main(int argc, char *argv[]) {
  ecs_world_t *ecs = ecs_init();

  ECS_COMPONENT(ecs, Position);
  ECS_COMPONENT(ecs, Velocity);

  ECS_SYSTEM(ecs, Move, EcsOnUpdate, Position, Velocity);

  ecs_entity_t e = ecs_insert(ecs,
    ecs_value(Position, {10, 20}),
    ecs_value(Velocity, {1, 2}));

  while (ecs_progress(ecs, 0)) { }
}
Same example in C++:

struct Position {
  float x, y;
};

struct Velocity {
  float x, y;
};

int main(int argc, char *argv[]) {
  flecs::world ecs;

  ecs.system<Position, const Velocity>()
    .each([](Position& p, const Velocity& v) {
      p.x += v.x;
      p.y += v.y;
    });

  auto e = ecs.entity()
    .insert([](Position& p, Velocity& v) {
      p = {10, 20};
      v = {1, 2};
    });

  while (ecs.progress()) { }
}
Projects using Flecs
If you have a project you'd like to share, let me know on Discord!