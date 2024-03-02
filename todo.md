## Engine
1. [x] Common camera buffer
2. [x] Make Mesh Asset
3. [x] Sprite size and scale decouple
4. [x] Fix Math
5. [x] Add Viewport, screenToWorld, worldToScreen 
6.  [x] Add mouse input
7.  [x] Input enums
8. [x] ECS improvement
9. [ ] Asset Path 
10. [ ] TileMap improvement
11. [ ] Think about render Asset. 
12. [ ] Add index_buffer and vertex_buffer into Gpu like texture_buffer
13. [ ] `BufferVec<T>` from bevy
14. [ ] Convert rgb to srgb in shader
15. [ ] Figure out why (-x, +y) or (+x, -y) scale is not working
16. [ ] Think about either I need common Vertex class or not
17. [ ] `Struct wgpu_util::DynamicBuffer`: A wgpu::Buffer which dynamically grows based on the contents.
18. [ ] Tween improvements
19. [ ] Input: add just pressed
20. [ ] Copy tests from bevy-ecs
21. [ ] Organize project structure
22. [ ] Fix QUIT


## Game
1. [x] Mouse based Road Placer
2. [ ] Fix road miss alignment when camera is moved

## Learn
1. [x] How shipyard does ecs with spare set. 
      - Afaik it puts same components in a vec and uses spare set inside archetypes. I think this will not help in cache locality as while iterating component vec it will jump back and forth inside array
  
2. [ ] How to use `std::ptr` in rust


# bevy RenderAsset
- So there is list of assets to be uploaded to gpu
- Every frame a system uploads stuff to gpu from it
- Figure out where Render asset is pushed inside that list
- Bevy have concept of sub app which is getting used
- I dont know who owns the sub app