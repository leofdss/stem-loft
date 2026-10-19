## Purpose

Traz os arquivos de stems de uma música para o projeto e os normaliza na entrada, para que a reprodução nunca tenha que lidar com áudio inconsistente.

## ADDED Requirements

### Requirement: Taxa de amostragem consistente
O importador MUST rejeitar com um erro explícito um stem cuja taxa de amostragem difere dos stems já presentes no projeto, e MUST NOT reamostrá-lo silenciosamente.

#### Scenario: Taxa de amostragem diferente
- **WHEN** um stem de 48 kHz é importado num projeto cujos stems são de 44,1 kHz
- **THEN** a importação é rejeitada com um erro que cita as duas taxas

### Requirement: Upmix de mono na importação
O importador SHALL fazer o upmix de um stem mono para estéreo no momento da importação, para que a reprodução só receba áudio estéreo.

#### Scenario: Stem mono
- **WHEN** um stem mono é importado
- **THEN** ele é guardado como estéreo, com o mesmo sinal nos dois canais

### Requirement: Duração lida do cabeçalho
O importador SHALL ler o durationSec de cada stem do cabeçalho do arquivo, sem decodificar o arquivo inteiro, e guardá-lo no projeto.

#### Scenario: Duração registrada
- **WHEN** um stem é importado
- **THEN** o projeto registra o durationSec dele conforme o cabeçalho do arquivo
