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

## Enfoque Integrado para la Selección de Arquitectura de Caché

Reconociendo las limitaciones de utilizar métricas individuales como el "miss
rate" o el tiempo de ejecución para comparar arquitecturas de caché, hemos
desarrollado una métrica combinada que ofrece una visión más completa del
rendimiento.

### La Métrica Combinada

Nuestra nueva métrica integra tres aspectos clave del rendimiento de la caché:

1. **Miss Ratio**: Representa la proporción de fallos de caché en relación con
   el total de accesos.
2. **Operaciones de Memoria**: Combina la cantidad de lecturas y escrituras en
   memoria.
3. **Tiempo de Ejecución**: Captura el tiempo total de ejecución de la
   simulación.

### Ventajas del Enfoque Integrado

1. **Equilibrio entre Precisión y Velocidad**: Al considerar tanto el miss
   ratio como el tiempo de ejecución, la métrica captura tanto la eficiencia de
   la caché como las penalizaciones temporales.

2. **Consideración de Patrones de Acceso**: Al incluir las operaciones de
   memoria, la métrica tiene en cuenta los patrones de lectura y escritura, que
   son cruciales para evaluar políticas como "Write Policy" y "Write Allocate
   Policy".

3. **Normalización Automática**: Los componentes de la métrica se normalizan
   automáticamente basándose en los valores máximos observados en todas las
   simulaciones, lo que permite una comparación justa entre diferentes
   configuraciones.

4. **Flexibilidad**: Esta métrica única puede utilizarse para optimizar todos
   los parámetros de la caché simultáneamente, incluyendo tamaño de caché,
   tamaño de bloque, división (split), Write Policy y Write Allocate Policy.

### Interpretación

Un valor más bajo de la métrica combinada indica un mejor rendimiento general.
La métrica busca minimizar simultáneamente el miss ratio, las operaciones de
memoria y el tiempo de ejecución, proporcionando una evaluación integral de la
eficiencia de la caché.

Esta aproximación integrada supera las limitaciones de usar métricas
individuales, ofreciendo una herramienta más robusta y versátil para la
selección y optimización de arquitecturas de caché.

## Consideraciones Adicionales

Para esta simulación, asumiremos que:

- El algoritmo LRU tiene un costo despreciable
- En los modos fully associative y set associative, el costo de buscar un
  bloque en la caché es despreciable

Es importante notar que estas suposiciones son específicas de la simulación. Al
tomar decisiones de implementación real, se deberían tener en cuenta los costos
efectivos de estas operaciones, tanto en términos de implementación como de
ejecución.

Para todas las respuestas se asume que se utiliza una caché **fully
associative**, ya que, según las consideraciones, el costo de buscar un bloque
en este tipo de caché es despreciable y el algoritmo LRU maximiza el uso de la
caché.

## ¿Cuál es la mejor arquitectura de cache para spice.trace?

```bash
grid_search combined_performance traces/spice.trace
```

| Cache Total Size | Cache Size | Block Size | Split | On Write Miss   | Write Policy | Combined Metric |
| ---------------- | ---------- | ---------- | ----- | --------------- | ------------ | --------------- |
| 4                | 1          | 1          | false | NoWriteAllocate | WriteThrough | 2.878020        |
| 16               | 2          | 2          | true  | NoWriteAllocate | WriteBack    | 1.423894        |
| 64               | 8          | 2          | false | NoWriteAllocate | WriteBack    | 0.711658        |
| 256              | 16         | 4          | true  | WriteAllocate   | WriteBack    | 0.297932        |
| 1024             | 16         | 16         | false | WriteAllocate   | WriteBack    | 0.119842        |
| 4096             | 128        | 8          | false | WriteAllocate   | WriteBack    | 0.025246        |
| 16384            | 512        | 8          | false | WriteAllocate   | WriteBack    | 0.003433        |
| 65536            | 64         | 256        | true  | WriteAllocate   | WriteBack    | 0.000285        |

Para el archivo de traza `spice.trace`, la mejor arquitectura de caché varía
según el tamaño total de la caché disponible:

- Para tamaños de caché pequeños (4, 16 y 64 bytes): Se observa un mejor
  rendimiento con la política "No Write Allocate" y "Write Through". Esto
  indica que, en estas configuraciones, la escritura directa en memoria
  principal (sin actualizar la caché) es más eficiente que la escritura en
  caché, especialmente cuando los datos escritos no se vuelven a leer pronto.
- Para tamaños de caché medianos y grandes (256, 1024, 4096, 16384 y 65536
  bytes): La mejor arquitectura utiliza la política "Write Allocate" y "Write
  Back". En este caso, al escribir en la caché y actualizar la memoria
  principal solo cuando se reemplaza un bloque, se reduce el número de accesos
  a memoria principal y se mejora el rendimiento.

En general, para `spice.trace` se observa una tendencia hacia arquitecturas
con "Write Allocate" y "Write Back" a medida que aumenta el tamaño de la
caché.

## ¿Cuál es la mejor arquitectura de cache para cc.trace?

```bash
grid_search combined_performance traces/cc.trace
```

| Cache Total Size | Cache Size | Block Size | Split | On Write Miss   | Write Policy | Combined Metric |
| ---------------- | ---------- | ---------- | ----- | --------------- | ------------ | --------------- |
| 4                | 1          | 1          | false | NoWriteAllocate | WriteThrough | 2.850054        |
| 16               | 2          | 2          | true  | NoWriteAllocate | WriteBack    | 1.428128        |
| 64               | 4          | 4          | true  | NoWriteAllocate | WriteBack    | 0.718400        |
| 256              | 8          | 8          | true  | WriteAllocate   | WriteBack    | 0.334571        |
| 1024             | 16         | 16         | false | WriteAllocate   | WriteBack    | 0.142906        |
| 4096             | 32         | 32         | false | WriteAllocate   | WriteBack    | 0.060836        |
| 16384            | 64         | 64         | false | WriteAllocate   | WriteBack    | 0.020179        |
| 65536            | 128        | 128        | false | WriteAllocate   | WriteBack    | 0.003575        |

Para el archivo de traza `cc.trace`, se observa un patrón similar al de
`spice.trace`:

- Tamaños de caché pequeños (4, 16 y 64 bytes): "No Write Allocate" y
  "Write Through" ofrecen el mejor rendimiento.
- Tamaños de caché medianos y grandes (256, 1024, 4096, 16384 y 65536
  bytes): "Write Allocate" y "Write Back" se convierten en la mejor opción.

Al igual que con `spice.trace`, `cc.trace` se beneficia de "Write Allocate" y
"Write Back" con caches más grandes.

## ¿Cuál es la mejor arquitectura de cache para tex.trace?

```bash
grid_search combined_performance traces/tex.trace
```

| Cache Total Size | Cache Size | Block Size | Split | On Write Miss   | Write Policy | Combined Metric |
| ---------------- | ---------- | ---------- | ----- | --------------- | ------------ | --------------- |
| 4                | 1          | 1          | false | NoWriteAllocate | WriteThrough | 2.781662        |
| 16               | 2          | 2          | true  | NoWriteAllocate | WriteBack    | 1.197989        |
| 64               | 4          | 4          | true  | WriteAllocate   | WriteBack    | 0.484909        |
| 256              | 8          | 8          | true  | WriteAllocate   | WriteBack    | 0.229445        |
| 1024             | 32         | 8          | true  | WriteAllocate   | WriteBack    | 0.002941        |
| 4096             | 16         | 64         | true  | WriteAllocate   | WriteBack    | 0.000462        |
| 16384            | 16         | 256        | true  | WriteAllocate   | WriteBack    | 0.000126        |
| 65536            | 16         | 1024       | false | WriteAllocate   | WriteBack    | 0.000033        |

El archivo de traza `tex.trace` muestra una preferencia aún más marcada por
"Write Allocate" y "Write Back":

- Incluso para el tamaño de caché más pequeño (4 bytes): "No Write
  Allocate" y "Write Through" son la mejor opción.
- Para todos los demás tamaños de caché (16, 64, 256, 1024, 4096, 16384 y
  65536 bytes): "Write Allocate" y "Write Back" ofrecen el mejor rendimiento,
  con una mejora significativa en la métrica combinada a medida que aumenta el
  tamaño de la caché.

`tex.trace` se beneficia especialmente de "Write Allocate" y "Write Back", lo
que sugiere un patrón de acceso a datos con mayor reutilización de
escrituras.

## ¿Por qué hay diferencias entre los programas? Si tuviera que diseñar un cache para un computador que ejecutará solo estos programas, que arquitectura usaría?

Las diferencias en las arquitecturas de caché óptimas para `spice.trace`,
`cc.trace` y `tex.trace` se deben a las variaciones en sus patrones de acceso
a memoria. Cada programa tiene una forma particular de acceder a los datos,
lo que influye en la eficiencia de diferentes estrategias de caché.

Algunos factores que pueden causar estas diferencias son:

- Localidad espacial: Programas con alta localidad espacial acceden a
  direcciones de memoria cercanas entre sí. Esto favorece a cachés con bloques
  más grandes, ya que se carga un conjunto de datos contiguos en cada acceso.
- Localidad temporal: Programas con alta localidad temporal acceden
  repetidamente a las mismas direcciones de memoria en un corto período de
  tiempo. Esto beneficia a cachés con políticas de reemplazo que priorizan los
  datos accedidos recientemente (como LRU).
- Frecuencia de lectura/escritura: La proporción de lecturas y escrituras
  influye en la eficiencia de las políticas "Write Allocate" y "Write
  Through/Write Back". Programas con muchas escrituras pueden beneficiarse de
  "Write Allocate", mientras que aquellos con predominio de lecturas pueden
  funcionar mejor con "Write Through".

Si tuviéramos que diseñar una caché para un computador que ejecutará solo
`spice.trace`, `cc.trace` y `tex.trace`, se podría optar por una arquitectura
que ofrezca un buen rendimiento en los tres casos.

Considerando las tendencias observadas en los experimentos, una buena opción sería:

- Caché fully associative: Maximiza la flexibilidad en la ubicación de los
  bloques y, según las consideraciones, el costo de búsqueda es despreciable.
- Tamaño de caché relativamente grande: A partir de 256 bytes, "Write
  Allocate" y "Write Back" se convierten en la mejor opción para los tres
  programas. Un tamaño mayor (por ejemplo, 4096 bytes o más) podría ofrecer un
  mejor rendimiento general.
- Política "Write Allocate" y "Write Back": Esta combinación es la más
  eficiente para los tres programas en cachés de tamaño mediano y grande.
- Tamaño de bloque adaptable: Si bien no se puede determinar un tamaño de
  bloque óptimo universal, un tamaño moderado (entre 8 y 64 palabras) podría
  ser un buen punto de partida.
