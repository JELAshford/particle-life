# Particle Life (🦀)
Particle Life: non-physical interactive system [(like this)](https://www.youtube.com/watch?v=scvuli-zcRc) which generates remarkably "biological" patterns and behaviours. Current implementation is CPU bound, using a `kdtree` from the [`kiddo`](https://github.com/sdd/kiddo) crate and parallelised with [`rayon`](https://github.com/rayon-rs/rayon) to efficiently compute interactions. 

Additional features include self-implemented "free-cam" that allows scrolling and zooming over the infinite plane to watch the patterns move. 

No systematic benchmark yet, but can comfortably run at 60FPS on my 2019 Macbook Pro with 10,000 particles. 
