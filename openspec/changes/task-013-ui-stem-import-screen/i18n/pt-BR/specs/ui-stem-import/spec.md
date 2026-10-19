## Purpose

A tela em que o usuário escolhe os arquivos de stems para importar no projeto.

## ADDED Requirements

### Requirement: A importação envia um Command na hora
Confirmar uma importação SHALL enviar o Command de importação na hora, sem fila no cliente e sem atualização otimista.

#### Scenario: Usuário confirma uma importação
- **WHEN** o usuário confirma os arquivos selecionados
- **THEN** o Command de importação é enviado e a lista de stems não muda até o núcleo responder

### Requirement: Controle desabilitado até a resposta
O controle que disparou a importação SHALL ficar desabilitado até chegar a resposta daquele Command.

#### Scenario: Esperando o núcleo
- **WHEN** um Command de importação está em andamento
- **THEN** o botão de importar fica desabilitado até a resposta chegar
