## Informe cache

### ¿Cuál es la mejor arquitectura de cache para spice.trace?

Para determinar la mejor arquitectura de cache para spice.trace, se realizaron
pruebas con distintas configuraciones de cache, variando la cantidad de sets,
la cantidad de bloques por set y el tamaño de los bloques. El tamaño total del cache
vario entre 1024, 4069 y 16384 bytes. La métrica que utilizaremos para
determinar la mejor arquitectura de cache es el **tiempo de ejecución**.

#### Cache de 1024 bytes.

| Block Size \[words\] | Block Size\[bytes\] | Cache Size | Tiempo de ejecución\[ms\] |
| -------------------- | ------------------- | ---------- | ------------------------- |
| 1                    | 4                   | 256        | 37.119905                 |
| 2                    | 8                   | 128        | 53.974605                 |
| 4                    | 16                  | 64         | 75.521205                 |
| 8                    | 32                  | 32         | 109.423205                |
| 16                   | 64                  | 16         | 170.417605                |
| 32                   | 128                 | 8          | 320.267205                |
| 64                   | 256                 | 4          | 719.150405                |
| 128                  | 512                 | 2          | 2412.641605               |
| 256                  | 1024                | 1          | 13498.990405              |

#### Cache de 4096 bytes.

| Block Size \[words\] | Block Size\[bytes\] | Cache Size | Tiempo de ejecución\[ms\] |
| -------------------- | ------------------- | ---------- | ------------------------- |
| 1                    | 4                   | 1024       | 16.025605                 |
| 2                    | 8                   | 512        | 17.255605                 |
| 4                    | 16                  | 256        | 19.813605                 |
| 8                    | 32                  | 128        | 25.884005                 |
| 16                   | 64                  | 64         | 47.288005                 |
| 32                   | 128                 | 32         | 112.177605                |
| 64                   | 256                 | 16         | 244.513605                |
| 128                  | 512                 | 8          | 514.747205                |
| 256                  | 1024                | 4          | 1714.056005               |
| 512                  | 2048                | 2          | 8639.031205               |
| 1024                 | 4096                | 1          | 52840.840005              |

#### Cache de 16384 bytes.

| Block Size \[words\] | Block Size\[bytes\] | Cache Size | Tiempo de ejecución\[ms\] |
| -------------------- | ------------------- | ---------- | ------------------------- |
| 1                    | 4                   | 4096       | 7.339305                  |
| 2                    | 8                   | 2048       | 7.496205                  |
| 4                    | 16                  | 1024       | 7.781605                  |
| 8                    | 32                  | 512        | 8.155205                  |
| 16                   | 64                  | 256        | 11.465605                 |
| 32                   | 128                 | 128        | 26.264005                 |
| 64                   | 256                 | 64         | 47.288005                 |
| 128                  | 512                 | 32         | 112.177605                |
| 256                  | 1024                | 16         | 244.513605                |
| 512                  | 2048                | 8          | 514.747205                |
| 1024                 | 4096                | 4          | 1714.056005               |
| 2048                 | 8192                | 2          | 8639.031205               |
| 4096                 | 16384               | 1          | 52840.840005              |
