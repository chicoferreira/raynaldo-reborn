#set text(font: "IBM Plex Sans", lang: "pt", region: "pt")
#set page(numbering: "1")

#set raw(lang: "rs")

#align(
  center,
  grid(
    row-gutter: 2em,
    grid(
      row-gutter: 0.7em,
      text(size: 2em, weight: "bold")[Pathtracer - Raynaldo],
      text(size: 1.2em)[Relatório de Visualização e Iluminação]
    ),
    grid(
      columns: 3,
      column-gutter: 6em,
      row-gutter: 0.5em,
      [Francisco Ferreira], [Ivan Ribeiro], [Júlio Pinto],
      [PG55942], [PG55950], [PG57883],
    )
  ),
)

#let todo = text.with(fill: red)

= Introdução

= Funcionalidades

== Janela Interativa com Pathtracer Progressivo e Multithreading

Como funcionalidade principal do trabalho, foi implementada uma janela interativa com pathtracer progressivo e multithreading.

#todo[meter imagem da UI]

=== Arquitetura da Aplicação

A aplicação é estruturada em torno de um loop principal que integra três componentes fundamentais: o sistema de janelas _winit_, o renderer GPU via _wgpu_, e a interface gráfica _egui_. Esta arquitetura permite manter uma separação clara entre a lógica de renderização e a interface do utilizador, garantindo responsividade mesmo durante operações computacionalmente intensivas.

O sistema _winit_ gere todos os eventos da janela (redimensionamento, input do utilizador, etc.), enquanto _wgpu_ fornece uma abstração moderna sobre APIs gráficas (Vulkan, Metal, DirectX 12) para transferência eficiente de dados para a GPU. A biblioteca _egui_ oferece uma interface de modo imediato que é renderizada diretamente sobre a textura do pathtracer.

=== Sistema de Renderização Progressiva

O núcleo do sistema de renderização progressiva reside na estrutura `RenderState`, que mantém um _canvas_ de acumulação onde cada pixel armazena um tuplo `(r_sum, g_sum, b_sum, sample_count)`. A cada _sample_ renderizado é somado o valor do pixel a cada canal de cor, e o contador de amostras é incrementado.

Aquando de uma chamada de renderização, o sistema começa a renderizar os píxeis da imagem. Este processamento é feito em pacotes de 10.000 píxeis, onde são renderizados em paralelo via uma biblioteca de _threads_ (_rayon_). Isto é repetido até que o sistema atinja o número de amostras por pixel desejado ou que tenha excedido o tempo de CPU dedicado por frame ao pathtracing. Este valor de tempo de CPU é configurável na UI do _egui_. Quanto maior for este valor, menos responsiva será a interface.

Quando o utilizador muda parâmetros de renderização, como a câmara, definições de renderização, etc., o _canvas_ de acumulação é restaurado. Para evitar que, por exemplo, ao mexer a câmara, os píxeis sejam renderizados pela mesma ordem, são alocadas ordens diferentes de renderização. No passo de renderização, o sistema primeiro consulta um array com o mesmo tamanho do _canvas_ mas com valores aleatórios únicos, e renderiza o pixel correspondente a cada índice. Este array é alternado entre $N$ arrays de ordem #footnote[Este valor foi escolhido de forma a que não se note a periodicidade da ordem de renderização, que é de 5 arrays.], para que o sistema possa alternar entre eles a cada mudança de definição. Isto dá a impressão de que a ordem de renderização é aleatória, no entanto, sendo altamente eficiente e não comprometendo acessos concorrentes a píxeis.


```rust
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

Depois disto, o _canvas_ é convertido para um array de bytes, onde é feito o tonemapping e a divisão das cores pelo número de amostras feitas nesse píxel, e então é transferido para a textura que reside na GPU.

A GPU irá renderizar a textura para o ecrã de acordo com o _shader_ que está a ser utilizado.

```wgsl
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

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

O _shader_ é composto por um _vertex shader_ que gera um triângulo de acordo com o índice do vértice, e um _fragment shader_ que amostra a textura no píxel correspondente.

=== Sistema de Interação

Como controlo da câmara foi implementado um sistema de controlo FPV, onde o utilizador pode mover a câmara com as teclas WASD para movimento horizontal e frontal, usando espaço e shift para subir e descer respetivamente. Ao premir o botão esquerdo do rato, é possível rodar a câmara em torno do seu eixo através de movimentos de rotação _yaw_ e _pitch_.

Com a interface do _egui_ é possível visualizar e/ou modificar os seguintes parâmetros:
- Posição da câmara
- Yaw e Pitch da câmara
- FOV da câmara
- Ângulo de desfoque da câmara
- Distância de foco da câmara
- Orçamento de tempo por frame
- Samples por pixel
- Profundidade máxima de raios
- Progresso da renderização
- Tonemapping
- Framerate
- Tempo de renderização

#todo[meter imagem da UI]

== Carregamento da cena

As cenas do pathtracer podem ser carregadas via um ficheiro TOML, que define as propriedades da câmara, o ambiente, as geometrias e os materiais.

=== Estrutura do Ficheiro de Configuração

Cada ficheiro de cena está organizado hierarquicamente em três secções principais que definem completamente o estado inicial da renderização.

#set raw(lang: "toml")
A configuração da câmara, especificada através da secção `[camera]`, estabelece a posição inicial do observador no espaço tridimensional, a orientação através de ângulos yaw e pitch, o field of view que determina a abertura angular da perspetiva, e parâmetros avançados de profundidade de campo incluindo focus_distance e defocus_angle para simulação de efeitos de bokeh e desfoque.

A iluminação ambiental é controlada através da secção `[environment]`, que permite especificar o comportamento da luz de fundo quando raios não intersetam nenhuma geometria da cena, suportando tanto cores sólidas uniformes através de `type = "color"` com uma especificação da cor, como gradientes procedurais semelhantes ao céu via `type = "sky"`.

As geometrias da cena são definidas através de múltiplas secções `[[geometry]]`, cada uma especificando um objeto tridimensional incluindo o seu tipo geométrico primitivo, propriedades de material físico, e parâmetros específicos como posição, orientação e escala. Também define as propriedades do material.


```toml
[camera]
position = [1.85, 1.85, -4.0]
yaw = 90.0
pitch = 0.0
fov = 60.0
focus_distance = 0.1
defocus_angle = 0.0

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

A deserialização deste ficheiro é feita via biblioteca _serde_ que simplifica completamente este trabalho, tendo que apenas definir as _structs_ que representam os dados do ficheiro.

=== Tipos de Geometria Suportados

O sistema suporta quatro tipos principais de geometria primitiva, cada um configurado através de parâmetros específicos no ficheiro TOML.

*Esferas* são definidas por dois parâmetros essenciais: `center` especifica a posição central da esfera no espaço tridimensional através de coordenadas `[x, y, z]`, e `radius` define o raio escalar que determina o tamanho da esfera.

```toml
[[geometry]]
type = "sphere"
center = [0.87, 0.5, 0.43]
radius = 0.5
material = "dielectric"
refractive_index = 1.5
```

*Quads* representam superfícies planares retangulares configuradas através de três vetores: `origin` define o ponto de partida da superfície, `u` especifica o primeiro vetor de orientação que determina uma das dimensões do retângulo, e `v` define o segundo vetor perpendicular que completa a definição da superfície.

```toml
[[geometry]]
type = "quad"
origin = [0.0, 0.0, 3.7]
u = [3.7, 0.0, 0.0]
v = [0.0, 0.0, -3.7]
material = "lambertian"
texture = "solid"
color = [0.73, 0.73, 0.73, 1.0]
```

*Mesh de Triângulos* podem ser carregadas de ficheiros OBJ externos ou definidas implicitamente. Para modelos OBJ, a configuração requer `mesh_type = "obj_file"` e `path` especificando o caminho para o ficheiro do modelo. Para definição implícita, utiliza-se `mesh_type = "implicit"` com arrays `verts` contendo coordenadas de vértices e `indices`.

```toml
[[geometry]]
type = "triangle_mesh"
mesh_type = "obj_file"
path = "assets/dragon8k.obj"
material = "metal"
albedo = [0.9, 0.8, 0.6, 1.0]
fuzziness = 0.1
```

*Caixas Orientadas* são configuradas através de quatro vetores: `origin` define o ponto de partida da caixa, enquanto `u`, `v` e `w` especificam os três vetores de orientação que determinam as dimensões nas três direções principais.

```toml
[[geometry]]
type = "box"
origin = [1.76, 0.0, 1.96]
u = [1.046, 0.0, -0.340]
v = [0.0, 2.2, 0.0]
w = [0.340, 0.0, 1.046]
material = "lambertian"
texture = "solid"
color = [0.73, 0.73, 0.73, 1.0]
```

=== Sistema de Materiais

O sistema suporta quatro tipos de materiais, cada um configurado através de parâmetros específicos no ficheiro TOML.

*Materiais Lambertianos* representam superfícies difusas e suportam três tipos de textura. Para texturas sólidas usa-se `texture = "solid"` com `color = [r, g, b, a]` especificando a cor RGBA. Para padrões de xadrez configura-se `texture = "checker"` com `color1` e `color2` definindo as duas cores alternantes e `scale` controlando o tamanho de cada quadrado do padrão. Para texturas de imagem utiliza-se `texture = "image"` com `image` especificando o caminho para o ficheiro de textura.

```toml
# Cor sólida
material = "lambertian"
texture = "solid"
color = [0.7, 0.3, 0.3, 1.0]

# Padrão de xadrez
material = "lambertian"
texture = "checker"
color1 = [0.2, 0.3, 0.1, 1.0]
color2 = [0.9, 0.9, 0.9, 1.0]
scale = 0.32

# Textura de imagem
material = "lambertian"
texture = "image"
image = "assets/img.png"
```

*Materiais Metálicos* são configurados através de `material = "metal"` com dois parâmetros: `albedo` define a cor base do metal em formato RGBA, e `fuzziness` controla a rugosidade da superfície com valores entre 0.0 (perfeitamente liso) e 1.0 (muito rugoso).

```toml
material = "metal"
albedo = [0.8, 0.8, 0.9, 1.0]
fuzziness = 0.1
```

*Materiais Dielétricos* representam materiais transparentes e requerem apenas `material = "dielectric"` e `refractive_index` que especifica o índice de refração do material (por exemplo, 1.5 para vidro, 1.33 para água).

```toml
material = "dielectric"
refractive_index = 1.5
```

*Materiais Emissivos* funcionam como fontes de luz e são configurados através de `material = "emissive"` com `color` definindo a cor de emissão em formato RGBA e `intensity` especificando a intensidade luminosa da fonte.

```toml
material = "emissive"
color = [1.0, 1.0, 1.0, 1.0]
intensity = 15.0
```

=== Carregamento de Modelos OBJ

O sistema usa a biblioteca _tobj_ para carregar modelos OBJ.

```rust
let (models, _materials) = tobj::load_obj(path, &LoadOptions::default())?;
```

Durante o carregamento, o sistema extrai posições de vértices, índices de triângulos e coordenadas de textura.

No entanto, o sistema não suporta carregamento de materiais a partir de ficheiros OBJ.

#set raw(lang: "rust")

== Tonemapping

Para melhorar as cores reproduzidas pelo pathtracer, foi implementado um sistema de tonemapping. Este sistema, como dito anteriormente, é aplicado no CPU antes da transferência para a textura da GPU.

O sistema suporta vários operadores de tonemapping predefinidos, que podem ser alterados na UI em tempo real. Adicionar novos tonemappers é simples, basta criar uma nova variante de `Tonemapper` e definir a função de conversão.

=== Tonemapping Reinhard

Um tonemapper simples que implementámos foi o Reinhard. Este tonemapper é um dos mais populares e é baseado na fórmula:

$
  V_i = C_i / (1 + C_i)
$

onde $C_i$ é um canal de cor $i$ (RGB) e $V_i$ é o valor final desse canal.

#todo[adicionar imagem do tonemapping reinhard]

=== Tonemapping Reinhard com Saturação H&K

Um tonemapper que melhora o Reinhard é o Reinhard com Saturação H&K. Este tonemapper é baseado na fórmula:

$
  R = L / (1 + L)
$

$
  "Sat" = (max(R, G, B) - min(R, G, B)) / max(R, G, B)
$

$
  A = R times (1 + 0.2 "Sat")
$

$
  "Scale" = A / L
$

$
  V_i = C_i times "Scale"
$

onde $R$, $G$ e $B$ são os canais de cor, $L$ é a luminância do pixel, $C_i$ é um canal de cor $i$ (RGB) e $V_i$ é o valor final desse canal.

#todo[adicionar imagem do tonemapping reinhard com saturação H&K]

== Estrutura de Aceleração (BVH) via _Embree_

Para geometrias complexas, como as meshes que requerem uma grande quantidade de triângulos, fazer o teste de interseção de um raio para cada um deles é demasiado custoso. Para resolver isto, recorremos à biblioteca _Embree_ da _Intel_, que implementa uma BVH para acelerar o teste de interseções para geometrias genéricas.

A BVH é uma árvore binária onde cada nó contém um bounding box que envolve os objetos que estão contidos nele. Para cada raio, é feito o teste de interseção com o bounding box do nó, e se o raio intersetar o bounding box, é feito o teste de interseção com os objetos que estão contidos no nó.

Para construir a BVH, é necessário fazer _"upload"_ dos dados para a estrutura de dados do _Embree_ e depois chamar a função de construção da BVH. Este _"upload"_, como não suporta todos os tipos de geometrias que implementámos, há um processo de conversão feito por nossa parte.

O _Embree_ suporta nativamente triângulos e esferas. Assim sendo, as esferas podem ser cobertas pelo _Embree_ sem necessidade de conversão. As meshes de triângulos igualmente, apenas necessitando de uma conversão de tipo de dados para o formato que o _Embree_ requer.

Para os _quads_, como não são suportados nativamente, são convertidos em dois triângulos. Com o mesmo raciocínio, para as _boxes_, são decompostas em 6 quads, ou seja em 12 triângulos.

Ao fazer o teste de interseção, as coordenadas _UV_ não são devolvidas corretamente pelo _Embree_. Desta forma, temos que fazer a conversão do ponto de interseção para as coordenadas _UV_ da geometria original.

Para as esferas, a conversão é feita através do ângulo das coordenadas esféricas. Para as meshes de triângulos, a conversão é feita através da interpolação baricêntrica. Para os _quads_ e _boxes_, a conversão é feita através da interpolação bilinear, onde no caso dos _boxes_ já temos a face que estamos a intersetar.

A implementação da BVH deu-nos ganhos de velocidade extremamente significativos, como se pode ver na tabela abaixo.

#figure(
  image("assets/dragao.png", width: 50%),
  caption: "Cena do dragão",
) <dragon>


Para uma cena do dragão (@dragon), com 5 _SPP_ e uma janela 800x600 a correr num Ryzen 7 7700X nas cenas `dragon8k.toml` (8000 triângulos) e `dragon80k.toml` (80000 triângulos).

#figure(
  table(
    columns: 3,
    [Tracer], [`dragon8k.toml`], [`dragon80k.toml`],
    [Naive], [36.5s], [355.6s],
    [Embree], [0.096s], [0.112s],
  ),
  caption: [Tempos de renderização para a cena do dragão],
)

Como podemos ver, com o uso da BVH, o tempo de renderização é diminuído drasticamente.

= Demonstrações

= Conclusão

