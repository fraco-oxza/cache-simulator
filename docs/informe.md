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



