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

### Requirement: Stems aceitos
O importador SHALL aceitar stems WAV em PCM inteiro ou float IEEE, simples ou WAVE_FORMAT_EXTENSIBLE, mono ou estéreo, e MUST rejeitar qualquer outro arquivo, ou um stem com mais de dois canais, com um erro explícito.

#### Scenario: Stem de quatro canais
- **WHEN** um WAV de quatro canais é importado
- **THEN** a importação é rejeitada com um erro que informa o número de canais

### Requirement: Importação tudo ou nada
O importador SHALL conferir todos os arquivos de uma importação antes de escrever qualquer um deles, e MUST NOT deixar parte de uma importação rejeitada na pasta do projeto.

#### Scenario: Um arquivo ruim entre bons
- **WHEN** uma importação tem um stem válido de 44,1 kHz e um stem de 48 kHz
- **THEN** nada é escrito na pasta do projeto

### Requirement: Stems nunca sobrescrevem arquivos existentes
Cada stem importado SHALL ser escrito de forma atômica em `<id>.wav` na pasta do projeto, com um id derivado do nome do arquivo e com sufixo para não colidir com nenhum stem do projeto nem com nenhum arquivo já presente na pasta.

#### Scenario: Mesmo nome duas vezes
- **WHEN** `Guitar.wav` é importado num projeto que já tem um stem `guitar`
- **THEN** ele é guardado como `guitar-2.wav` e o arquivo existente fica intacto
