# Informe Cache

A continuación se analizará el comportamiento de la caché en base a algunos
archivos de trazas de ejecución de programas. Se solicita indicar la mejor
arquitectura para cada archivo, por lo tanto, antes de responder a las
preguntas, aclararemos cómo obtendremos la "mejor arquitectura".

## Metodología de Comparación

Para poder comparar arquitecturas de manera justa, utilizaremos un tamaño total
de caché equivalente dentro de las pruebas. En los parámetros se puede
especificar el tamaño de la caché y el tamaño de bloque en palabras. De esta
manera, podemos calcular el tamaño total de la caché de la siguiente manera:

$$ttc = cs \times bs \times ws$$

Donde:

- $ttc$: Tamaño total de la caché
- $cs$: Cantidad de bloques de caché
- $bs$: Tamaño de un bloque en palabras
- $ws$: Tamaño de una palabra en bytes

Se establecerán los siguientes valores para los tamaños totales de caché:

- 512 bytes
- 1024 bytes
- 2048 bytes
- 4096 bytes
- 8192 bytes
- 16384 bytes
- 32768 bytes
- 65536 bytes

Se utilizan varios tamaños de caché para tener una mejor idea de cómo se
comporta la caché en función de su capacidad.

Para cada uno de estos tamaños totales, se probarán todas las arquitecturas
posibles y se obtendrá el mejor resultado en base a una métrica seleccionada.

## Herramienta de Experimentación

Para realizar estos experimentos, se utiliza un binario específico llamado
`grid_search.rs` que realiza la búsqueda de la mejor arquitectura en base a una
métrica seleccionada.

## Selección de Métrica

Inicialmente se planteó utilizar el tiempo de ejecución como métrica principal;
sin embargo, de manera experimental se demostró que no es una buena métrica
para nuestro simulador. En cada ejecución de pruebas, el mejor resultado venía
dado por un tamaño de bloque de 1 y un tamaño de caché igual al tamaño total
disponible.

La explicación de este comportamiento se basa en cómo funciona el simulador de
caché: cuando utilizamos un tamaño de bloque mayor, cada vez que hay un fallo
(miss), la penalización de tiempo se basa en la cantidad de palabras que
tenemos que leer o escribir desde la memoria. Por lo tanto, a mayor tamaño de
bloque, mayor penalización de tiempo. Por otro lado, si utilizamos un tamaño de
bloque de 1, la penalización de tiempo es mínima, ya que solo se lee o escribe
una palabra.

### Ejemplo Comparativo

Para ilustrar mejor lo anteriormente mencionado, se mostrará un ejemplo:

#### Escenario 1: Caché con 64 bloques y tamaño de bloque de 4 palabras

Configuración: fully associative, write-back, write-allocate.

```
0 ffff0010 MISS, lectura de 4 palabras, penalización de 400 ns + 5 ns
0 ffff0011 HIT, penalización de 5 ns
0 ffff0012 HIT, penalización de 5 ns
0 ffff0013 HIT, penalización de 5 ns
0 ffff0010 HIT, penalización de 5 ns
0 ffff0011 HIT, penalización de 5 ns
0 ffff0012 HIT, penalización de 5 ns
0 ffff0013 HIT, penalización de 5 ns
```

Penalización total: 420 ns

#### Escenario 2: Mismo caso con tamaño de bloque de 1 palabra

```
0 ffff0010 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 ffff0011 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 ffff0012 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 ffff0013 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 ffff0010 HIT, penalización de 5 ns
0 ffff0011 HIT, penalización de 5 ns
0 ffff0012 HIT, penalización de 5 ns
0 ffff0013 HIT, penalización de 5 ns
```

Penalización total: 420 ns

Este es el caso más favorable cuando tenemos un tamaño de bloque mayor, y aun
así no hay ventaja. La situación empeora en otros escenarios:

#### Escenario 3: Accesos dispersos con tamaño de bloque de 4 palabras

```
0 fff10010 MISS, lectura de 4 palabras, penalización de 400 ns + 5 ns
0 fff20010 MISS, lectura de 4 palabras, penalización de 400 ns + 5 ns
0 fff30010 MISS, lectura de 4 palabras, penalización de 400 ns + 5 ns
0 fff40010 MISS, lectura de 4 palabras, penalización de 400 ns + 5 ns
0 fff10010 HIT, penalización de 5 ns
0 fff20011 HIT, penalización de 5 ns
0 fff30012 HIT, penalización de 5 ns
0 fff40013 HIT, penalización de 5 ns
```

Penalización total: 1640 ns

#### Escenario 4: Mismo caso con tamaño de bloque de 1 palabra

```
0 fff10010 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 fff20010 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 fff30010 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 fff40010 MISS, lectura de 1 palabra, penalización de 100 ns + 5 ns
0 fff10010 HIT, penalización de 5 ns
0 fff20011 HIT, penalización de 5 ns
0 fff30012 HIT, penalización de 5 ns
0 fff40013 HIT, penalización de 5 ns
```

Penalización total: 440 ns

Este patrón se puede generalizar para cualquier configuración de caché. Puede
verificarlo ejecutando:

```bash
grid_search execution_time traces/cc.trace
```

## Enfoque Mixto para la Selección de Arquitectura

Dado que la métrica de tiempo no es la mejor para comparar arquitecturas, se
utilizará principalmente la métrica de "miss rate" (tasa de fallos), que es la
proporción de fallos que tiene la caché en relación con la cantidad total de
accesos. Esta métrica demostró ser más adecuada para comparar arquitecturas,
aunque también tiene limitaciones.

La principal limitación es que no toma en cuenta las penalizaciones de tiempo,
por lo tanto, es incapaz de determinar los parámetros óptimos para la "Write
Policy" y "Write Allocate Policy". Por esto, se utilizará un enfoque mixto:

1. Se usará la métrica de "miss rate" para encontrar la mejor combinación de
   tamaño de caché, tamaño de bloque y división (split).
2. Luego se utilizará la métrica de "tiempo de ejecución" para determinar la
   mejor combinación de "Write Policy" y "Write Allocate Policy".

## Consideraciones Adicionales

Para esta simulación, asumiremos que:

- El algoritmo LRU tiene un costo despreciable
- En los modos fully associative y set associative, el costo de buscar un
  bloque en la caché es despreciable

Es importante notar que estas suposiciones son específicas de la simulación. Al
tomar decisiones de implementación real, se deberían tener en cuenta los costos
efectivos de estas operaciones, tanto en términos de implementación como de
ejecución.

## CC

| Cache Total Size | Cache Size | Block Size | Split | Tiempo (ms) | Miss Ratio |
| ---------------- | ---------- | ---------- | ----- | ----------- | ---------- |
| 4                | 1          | 1          | false | 113.30      | 1.0        |
| 8                | 1          | 2          | false | 170.77      | 0.741      |
| 16               | 1          | 4          | false | 263.40      | 0.611      |
| 32               | 2          | 4          | true  | 243.15      | 0.441      |
| 64               | 2          | 8          | true  | 343.01      | 0.333      |
| 128              | 2          | 16         | true  | 514.51      | 0.270      |
| 256              | 4          | 16         | false | 430.97      | 0.215      |
| 512              | 8          | 16         | false | 350.63      | 0.162      |
| 1024             | 8          | 32         | false | 448.95      | 0.110      |
| 2048             | 16         | 32         | false | 340.07      | 0.0756     |
| 4096             | 16         | 64         | false | 425.40      | 0.0508     |
| 8192             | 32         | 64         | false | 320.87      | 0.0342     |
| 16384            | 64         | 64         | false | 225.23      | 0.0191     |
| 32768            | 128        | 64         | false | 154.60      | 0.0079     |
| 65536            | 128        | 128        | false | 149.08      | 0.0035     |

## Spice

| Cache Total Size | Cache Size | Block Size | Split | Tiempo (ms) | Miss Ratio |
| ---------------- | ---------- | ---------- | ----- | ----------- | ---------- |
| 4                | 1          | 1          | false | 111.65      | 1.0        |
| 8                | 1          | 2          | false | 171.27      | 0.729      |
| 16               | 1          | 4          | false | 270.28      | 0.617      |
| 32               | 2          | 4          | true  | 245.39      | 0.453      |
| 64               | 2          | 8          | true  | 357.39      | 0.355      |
| 128              | 2          | 16         | true  | 532.86      | 0.283      |
| 256              | 4          | 16         | false | 389.34      | 0.188      |
| 512              | 8          | 16         | true  | 298.91      | 0.128      |
| 1024             | 8          | 32         | true  | 378.34      | 0.0879     |
| 2048             | 8          | 64         | false | 470.56      | 0.0580     |
| 4096             | 128        | 8          | false | 122.23      | 0.0244     |
| 8192             | 64         | 32         | false | 132.16      | 0.0087     |
| 16384            | 512        | 8          | false | 107.42      | 0.0034     |
| 32768            | 128        | 64         | false | 109.79      | 0.0008     |
| 65536            | 64         | 256        | true  | 111.66      | 0.0003     |

## Tex

| Cache Total Size | Cache Size | Block Size | Split | Tiempo (ms) | Miss Ratio |
| ---------------- | ---------- | ---------- | ----- | ----------- | ---------- |
| 4                | 1          | 1          | false | 97.86       | 1.0        |
| 8                | 1          | 2          | false | 141.91      | 0.780      |
| 16               | 2          | 2          | true  | 126.98      | 0.583      |
| 32               | 2          | 4          | true  | 198.28      | 0.417      |
| 64               | 4          | 4          | true  | 162.27      | 0.289      |
| 128              | 4          | 8          | true  | 212.76      | 0.212      |
| 256              | 4          | 16         | true  | 290.83      | 0.162      |
| 512              | 32         | 4          | false | 88.78       | 0.0047     |
| 1024             | 32         | 8          | true  | 88.89       | 0.0024     |
| 2048             | 16         | 32         | false | 89.04       | 0.0006     |
| 4096             | 16         | 64         | true  | 89.08       | 0.0003     |
| 8192             | 16         | 128        | true  | 89.11       | 0.0002     |
| 16384            | 16         | 256        | true  | 89.25       | 0.0001     |
| 32768            | 16         | 512        | true  | 89.56       | 0.00005    |
| 65536            | 16         | 1024       | false | 89.35       | 0.00002    |

**Observation:**

Similar to the spice.trace, the cc.trace also shows a trend where larger cache sizes with smaller block sizes perform better for larger total cache sizes. However, the difference is not as pronounced as in the spice.trace. This suggests that the cc.trace might have a balance of spatial and temporal locality.

**General Conclusion:**

Across all three traces (tex, spice, and cc), increasing the total cache size generally leads to a decrease in the miss ratio and, consequently, better performance. However, the optimal block size depends on the specific access patterns of the trace. Traces with more spatial locality (like spice) benefit from smaller block sizes with larger caches, while traces with a balance of spatial and temporal locality (like cc) might show a less pronounced preference for smaller block sizes.
