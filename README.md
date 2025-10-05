# Proyecto 2: Raytracing


Este proyecto es un **raytracer en Rust** que renderiza una escena 3D hecha con bloques  al estilo *Minecraft*.  
Soporta varios materiales (tierra, madera, hojas, agua y minerales) con efectos de **iluminación difusa, especular, reflexión y refracción**.


## [🎥 Video Demostrativo](https://youtu.be/EGLqN1GQTAo) 


---


## 🛠️ Requisitos

- [Rust](https://www.rust-lang.org/) (>= 1.70 recomendado)
- Dependencias:
  - [`nalgebra-glm`](https://crates.io/crates/nalgebra-glm) – álgebra lineal.
  - [`minifb`](https://crates.io/crates/minifb) – ventana y framebuffer.

Instálalas con:

```bash
cargo add nalgebra-glm
cargo add minifb
```

---

## ▶️ Ejecución

Clona el repositorio y compila:

```bash
git clone https://github.com/can23384/Proyecto-2-Raytracing
cd Proyecto-2-Raytracing
cargo run --release
```

⚡ **Recomendación**: usa `--release` para obtener mejor rendimiento al renderizar.

Se abrirá una ventana donde podrás explorar la escena.


---

## 🎮 Controles

- ⬅️ / ➡️ → Rotar cámara en horizontal.  
- ⬆️ / ⬇️ → Rotar cámara en vertical.  
- `W` → Acercar cámara (zoom in).  
- `S` → Alejar cámara (zoom out).  
- `Esc` → Salir.

---

## 🏞️ Escena

La escena actual contiene:

- **Piso** de tierra con un **río** en el centro y un **lago** conectado.  
- Varias **montañas** escalonadas y colinas pequeñas.  
- **Árboles** con tronco de madera y copa de hojas.  
- Vetas de **Mineral** incrustadas en el terreno.

---

## 📂 Estructura del proyecto

```
src/
├── main.rs          # bucle principal y render
├── framebuffer.rs   # gestión del framebuffer
├── ray_intersect.rs # definición de rayos e intersecciones
├── block.rs         # definición de bloques (AABB)
├── color.rs         # manejo de colores
├── camera.rs        # cámara orbital y zoom
├── light.rs         # fuente de luz
├── material.rs      # materiales y propiedades
```

---

## ⚙️ Materiales 

Materiales usados en el proyecto:

- **Tierra** → opaco, sin reflexión ni refracción.  
- **Madera** → opaco, sin reflexión ni refracción.  
- **Hojas** → mayormente difusas, con un poco de **reflexión**.  
- **Agua** → transparente, con **reflexión y refracción**.  
- **Mineral** → metálico, con **reflexión**, sin refracción.  
---
