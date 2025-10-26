# ndnm-libs: A Caixa de Ferramentas Essencial do NDNM

## 🧬 O DNA do Projeto

`ndnm-libs` (anteriormente `ndnm-core`) é o coração compartilhado do ecossistema NDNM. Ele **NÃO é um executável**, mas sim uma **biblioteca Rust (`--lib`)** que fornece os blocos de construção fundamentais, tipos comuns e utilitários essenciais usados por todos os outros módulos principais (`ndnm-brazil`, `ndnm-hermes`, `ndnm-exdoida`) e pelos nodes individuais (`nodes/*`).

Pense nele como a caixa de ferramentas definitiva do projeto: se algo precisa ser compartilhado ou padronizado entre diferentes partes do backend, **deve** residir aqui. O objetivo é máximo reuso de código e mínima duplicação.

## 🎯 Principais Responsabilidades e Conteúdos (V2 - Nova Config)

Este crate define e exporta:

1.  **`trait Node`:** A interface (contrato) fundamental que *todo* node executável Rust deve implementar. Define a estrutura básica de `validate` (validação síncrona/rápida) e `process` (execução assíncrona/principal). Nodes em outras linguagens devem *conceitualmente* aderir a essa estrutura.
2.  **`AppError` Enum:** O tipo de erro padronizado para todo o sistema, facilitando o tratamento de erros e a comunicação entre módulos. Inclui variantes comuns como `BadRequest` e `Internal`.
3.  **Estruturas de Configuração Avançadas (V2):** Define a **nova estrutura detalhada** para os arquivos `config.yaml` dos nodes, permitindo interfaces dinâmicas complexas. Isso inclui:
    * **`NodeConfig` (raiz):** Contém `node_id_hash`, `label`, `node_type`, `sections` e `input_fields`. **Não contém mais `port`, `inputs_mode`, `outputs_mode`, etc.**
    * **`Section`:** Define um grupo de slots de I/O com um `behavior` específico (ex: `"auto_increment"`, `"dynamic_per_file"`).
    * **`SlotTemplate`:** Descreve um par `input`/`output` vinculado dentro de uma seção, incluindo `name` (base), `label` (pode ter placeholders), `type` (ex: `"FILE_CONTENT"`, `"STRING"`, `"NUMBER"`), e `connections` (`1` ou `"n"`).
    * **`InputFieldConfig`:** Define os controles internos do node (ex: `type: "text"`, `type: "button"`).
    * **Função `load_config`:** Atualizada para parsear e validar essa nova estrutura YAML.
4.  **Utilitários de Servidor (`router`, `serve` - Potencialmente Simplificados):** As funções genéricas para criar um servidor Axum básico podem ser mantidas, mas a responsabilidade de *como* o node é exposto (qual porta, etc.) agora é primariamente do `ndnm-hermes`. Essas funções podem se tornar helpers internos para `hermes` ou para nodes Rust que *optem* por rodar standalone (para testes).
5.  **Runner Genérico (`run_node` - Potencialmente Simplificado/Removido):** Com `hermes` gerenciando os processos/portas, um runner genérico que *assume* que o node controla sua própria porta pode se tornar obsoleto ou precisar ser adaptado para um contexto diferente (ex: testes locais).
6.  **Tipos Comuns (Futuro):** Continua sendo o local para tipos compartilhados (tensores, mensagens inter-serviços, etc.).

## 🚫 O Que NÃO Pertence Aqui (Sem Alterações Significativas)

* Lógica específica de um node (`process`).
* Orquestração de fluxo (`ndnm-hermes`).
* Comunicação com frontend (`ndnm-brazil`).
* Coleta de logs/métricas (`ndnm-exdoida`).
* Código específico de linguagem externa.

## 🤝 Como Usar

Outros crates Rust dentro do workspace `ndnm-backend` usarão `ndnm-libs` para acessar as definições de `Node`, `AppError`, e **crucialmente, as novas structs de configuração** para parsear os `config.yaml` e entender a estrutura dos nodes.

```toml
[dependencies]
ndnm-libs = { path = "../ndnm-libs" }
