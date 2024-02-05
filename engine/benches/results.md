
Results from `cargo bench` on bp.
brot2 comparisons used `-O3 -Ofast`.

# Fractal

For Original:

| commit-id | mean    | slope    | tile     | Notes |
| :-------: |    ---: |  ---:    | ---:     | ---   |
| be74f12   |  4.43ns |  5.52ns  | 4.044ms  | brot2 mean is 2.002ns (double) |

For Mandelbrot3:

| commit-id |  mean  | slope  |
| :-------: | ---:   | ---:   |
| be74f12   | 5.13ns | 6.37ns |

# Palette

For Mandy:

| commit-id | pixel mean | tile   | Notes |
| :-------: | ---:       | ---:   | ---   |
| be74f12   | 42.85ns    | 3.50ms | brot2 means 45.05ns/pixel, 4.1ms/tile - great result! |
