#import "@preview/typslides:1.2.6": *

#show: typslides.with(
  ratio: "16-9",
  theme: "bluey",
)

#set text(lang: "pt", region: "pt", font: "IBM Plex Sans")
#set par(justify: true)

#set raw(lang: "rs")

#front-slide(
  title: [Visualização e Iluminação],
  subtitle: [Apresentação Trabalho Prático],
  authors: [Ivan Ribeiro, Francisco Ferreira e Júlio Pinto],
)

#show link: underline
#show link: text.with(fill: blue)

#title-slide[
  #set par(justify: false)

  = _Pathtracer_ - Raynaldo
]

#slide(title: [Tecnologia])[
  - *_Rust_*
  - *_WGPU_*
  - *_Winit_*
  - *_Egui_*
  - *_Embree_ (_Intel_)*

  Segurança, _performance_, controlo sobre a _pipeline_ de renderização, _APIs_ modernas
]

#slide(title: [Funcionalidades Implementadas])[
  - _Multithreading_
  - Carregamento de cenas de _OBJ_
  - Janela interativa com _pathtracer_ progressivo
  - _Tonemapping_
  - Estrutura de aceleração (_BVH_)
  - #strike[_Participating Media_]
]

#slide(title: [Arquitetura da Aplicação])[
  Três componentes fundamentais integrados:

  - *_Winit_*: Sistema de janelas e eventos
  - *_WGPU_*: _Renderer_ na _GPU_ (_Vulkan_/_Metal_/_DirectX_ 12)
  - *_Egui_*: _Interface_ gráfica de modo imediato
  - *_Renderer_*: _Output_ do _raytracer_
]

#slide(title: [Sistema de Renderização Progressiva e Paralela])[
  ```rs
  struct RenderState {
    canvas: AccumulationCanvas, // (r_sum, g_sum, b_sum, sample_count)
    current_render_pixel: usize,
    pixel_orders: [Vec<usize>; 5], // Ordem aleatória de renderização
  }
  ```

  - Renderização em lotes de 10.000 píxeis
  - Processamento paralelo via *_Rayon_*
  - Orçamento de tempo por _frame_ configurável
  - 5 ordens de renderização diferentes para evitar padrões
]

#slide(title: [Sistema de Renderização Progressiva e Paralela])[
  #set text(size: 0.9em)
  ```rs
  while !state.render_state.is_finished(state.samples_per_pixel)
      && instant.elapsed() < Duration::from_millis(state.time_budget_ms)
  {
      // Processamento paralelo de lote de pixels
      state.render_state.get_current_pixel_order()
          [state.render_state.current_render_pixel..end]
          .par_iter()
          .for_each(|&index_in_buffer| {
                let x = index_in_buffer as u32 % width;
                let y = index_in_buffer as u32 / width;
                let color = state.scene.render_sample(x, y, state.max_ray_depth);

                state.render_state.canvas.increment_pixel(x, y, color);
          });
  }
  ```
]

#slide(title: [_GPU Shader Pipeline_])[
  ```wgsl
  @vertex
  fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
      // Gerar triângulo de acordo com o índice do vértice
      let x: f32 = f32((vertex_index << 1u) & 2u) * 2.0 - 1.0;
      let y: f32 = f32(vertex_index & 2u) * 2.0 - 1.0;
      out.position = vec4<f32>(x, y, 0.0, 1.0);
      out.uv = (vec2<f32>(x, y) + vec2<f32>(1.0)) * 0.5;
      return out;
  }

  @fragment
  fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
      return textureSample(tex, tex_sampler, input.uv);
  }
  ```
]

#slide(title: [Sistema de Interação])[


  #set text(size: 0.9em)

  #grid(
    columns: 2,
    [
      *Controlo da Câmara (_FPV_):*
      - _WASD_: movimento horizontal e frontal
      - Espaço/_Shift_: subir/descer
      - _Mouse_: rotação _yaw_/_pitch_

      *_Interface_ _Egui_:*
      - Posição e orientação da câmara
      - _FOV_ e parâmetros de desfoque
      - _Samples_ por pixel e profundidade de raios
      - _Tonemapping_ e estatísticas de _performance_
    ],
    [
      #image("assets/ui.png", width: 100%)
    ],
  )
]

#slide(title: [Carregamento de Cenas - _TOML_])[
  ```toml
  [camera]
  position = [1.85, 1.85, -4.0]
  yaw = 90.0
  pitch = 0.0
  fov = 60.0

  [environment]
  type = "sky"

  [[geometry]]
  type = "sphere"
  center = [0.0, 0.0, 0.0]
  radius = 1.0
  material = "lambertian"
  texture = "solid"
  color = [0.7, 0.3, 0.3, 1.0]
  ```
]

#slide(title: [Tipos de Geometria Suportados])[
  - *Esferas*: _center_ + _radius_
  - *_Quads_*: _origin_ + vetores u, v
  - *Malhas de Triângulos*: ficheiros _OBJ_ ou definição implícita
  - *Caixas Orientadas*: _origin_ + vetores u, v, w

  Deserialização automática via *_Serde_*
]

#slide(title: [Sistema de Materiais])[
  - *Lambertiano*: superfícies difusas (cor sólida, xadrez, texturas)
  - *Metálico*: _albedo_ + _fuzziness_ (rugosidade)
  - *Dielétrico*: materiais transparentes (índice de refração)
  - *Emissivo*: fontes de luz (cor + intensidade)
]

#slide(title: [_Tonemapping_])[
  *_Reinhard_ Simples:*
  $V_i = C_i / (1 + C_i)$

  *_Reinhard_ com Saturação H&K:* Modificação do _Reinhard_

  Aplicado na _CPU_ antes da transferência para _GPU_
]

#slide(title: [_BVH_ via _Embree_ (_Intel_)])[
  *Problema:* Teste de interseção para cada triângulo é custoso \
  *Solução:* Estrutura de aceleração _BVH_ via _Embree_


  _Embree_ já tem suporte nativo para esferas e _meshes_ de triângulos

  *No entanto,* não tem suporte para _quads_ e _boxes_.\
  *Solução:* Fazer conversão de _quads_ e _boxes_ para _meshes_ de triângulos.

  *Outro problema:* _Embree_ não devolve coordenadas _UVs_ corretamente. \
  *Solução:* Calcular _UVs_ manualmente a partir do ponto de interseção.

  *Ganhos de _Performance_:*
  - _Dragon_ 8k: 36.5s → 0.096s
  - _Dragon_ 80k: 355.6s → 0.112s
]

#slide(title: [Demonstrações - Resultados])[
  #grid(
    columns: 2,
    column-gutter: 1em,
    [
      #figure(
        image("assets/demo_balls.png", width: 100%),
        caption: [Cena com esferas],
      )
    ],
    [
      #figure(
        image("assets/demo_dragon.png", width: 100%),
        caption: [Dragão de _Stanford_],
      )
    ],
  )

  #align(center)[
    Todas as imagens renderizadas com 2000 _SPP_
  ]
]

#slide(title: [_Cornell Box_])[
  #grid(
    columns: 2,
    column-gutter: 1em,
    [
      #figure(
        image("assets/demo_cornell_box.png", width: 100%),
        caption: [_Cornell Box_ clássica],
      )
    ],
    [
      #figure(
        image("assets/demo_cornell_box_images.png", width: 100%),
        caption: [_Cornell Box_ com texturas],
      )
    ],
  )
]

#slide(title: [Conclusão])[
  *Objetivos Alcançados:*
  - _Pathtracer_ funcional e eficiente em _Rust_
  - _Interface_ interativa com renderização progressiva
  - Sistema completo de materiais e geometrias
  - Aceleração significativa com _BVH_

  *Trabalho Futuro:*
  - Suporte completo para materiais _OBJ_
  - Redução de ruído
  - Edição de mundo na _UI_
  - _Export_ de imagens
]

#title-slide[
  Demonstração
]
