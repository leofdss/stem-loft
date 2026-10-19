## Purpose

Guarda o volume, o mute e o solo de cada stem e aplica as mudanças na reprodução em tempo real.

## ADDED Requirements

### Requirement: Mudanças de mixer seguras para tempo real
Uma mudança de volume, mute ou solo SHALL chegar à reprodução por estado compartilhado sem lock; o callback de áudio MUST NOT bloquear num lock para lê-la.

#### Scenario: Fader movido durante a reprodução
- **WHEN** o usuário muda o volume de um stem enquanto a música toca
- **THEN** o volume muda no próximo bloco de áudio sem falha

### Requirement: Mute e solo
Mutar um stem SHALL silenciá-lo, e quando algum stem está em solo só os stems em solo SHALL ser audíveis.

#### Scenario: Solo em um stem
- **WHEN** um stem é colocado em solo
- **THEN** só aquele stem fica audível até o solo ser desfeito

### Requirement: Mudanças do mixer persistem pelo checkpoint
Uma mudança no mixer SHALL marcar o projeto como sujo para o checkpoint com debounce, em vez de gravar o projeto a cada mudança.

#### Scenario: Arrastando um fader
- **WHEN** um fader é arrastado por vários valores
- **THEN** o projeto é marcado como sujo e gravado uma vez no próximo checkpoint
