# ndnm-hermes: O Maestro Orquestrador do NDNM

## 🏛️ O Cérebro da Operação (Control Plane - V2)

`ndnm-hermes` é o componente central responsável pela **descoberta, gerenciamento e orquestração** dos nodes (`nodes/*`) no ecossistema NDNM. Ele atua como o **Control Plane** do backend, recebendo comandos de alto nível do `ndnm-brazil` (BFF) e traduzindo-os em ações concretas envolvendo os nodes individuais.

Sua responsabilidade crucial é interpretar as **configurações avançadas dos nodes** (definidas nos `config.yaml`, com `sections`, `behaviors` e `slot_templates`) e gerenciar o fluxo de execução e de dados (que podem variar de simples strings a Gigabytes) entre eles.

## 🎯 Principais Responsabilidades (Detalhado)

1.  **Descoberta e Gerenciamento de Nodes:**
    * **Descoberta:** Escaneia o diretório `nodes/`, identifica nodes válidos e **parseia a estrutura completa do `config.yaml`** (incluindo `node_id_hash`, `sections`, `behaviors`, `slot_template` com `input`/`output` pareados, `connections`, `input_fields`) usando `ndnm-libs`. Valida a estrutura contra o schema esperado.
    * **Registro Detalhado:** Mantém um registro interno com a **estrutura completa** de cada node (não apenas tipos básicos), incluindo as definições de todas as seções e templates de slots.
    * **Gerenciamento de Portas/Comunicação:** **Define e gerencia** o mecanismo de comunicação com cada node (ex: alocando portas HTTP, gerenciando sub-processos, etc.), já que a porta não é mais definida no node.
    * **Disponibilização de Configs (Estrutura Rica):** Fornece a **estrutura detalhada** dos nodes (com seções, slots, etc.) para o `ndnm-brazil`, permitindo que o `ndnm-argos` renderize interfaces dinâmicas complexas.
2.  **Orquestração de Fluxo (Execução do Grafo):**
    * **Recepção e Análise do Grafo:** Recebe a definição do grafo do `ndnm-brazil` e a valida contra o registro de nodes e suas estruturas de slots (nomes, tipos, limites de conexão).
    * **Gerenciamento de Handles Dinâmicos:** Interpreta os `behaviors` das seções (`auto_increment`, `dynamic_per_file`, etc.). Para `dynamic_per_file`, por exemplo, interage com o node correspondente (ou com o sistema de arquivos via node) para determinar quais slots dinâmicos precisam ser instanciados antes da execução. **Informa `ndnm-brazil` sobre quaisquer mudanças na estrutura dinâmica dos slots** para atualização da UI.
    * **Execução Sequencial/Paralela:** Determina a ordem de execução dos nodes baseado nas conexões do grafo, gerenciando dependências. (Pode otimizar para execução paralela onde possível).
    * **Comunicação com Nodes:** Envia requisições para os endpoints corretos dos nodes (ex: `POST /run`) com os dados de entrada formatados. **Responsável por lidar com a transferência de dados potencialmente grandes** (ex: tensores, imagens, conteúdos de arquivos), possivelmente usando estratégias como streaming, referências a arquivos temporários, ou comunicação direta entre nodes se a arquitetura permitir.
    * **Mapeamento de Dados:** Garante que a saída de um `output handle` (ex: `copied_output_0`) seja corretamente mapeada e enviada para o `input handle` conectado (ex: `some_input_3`). Lida com a nomenclatura dinâmica (ex: `internal_output_nomedoarquivo.txt`).
    * **Tratamento de Erros:** Captura erros dos nodes, interrompe fluxos dependentes e reporta o erro detalhado para o `ndnm-brazil`.
3.  **Interface com `ndnm-brazil`:**
    * **API Robusta:** Expõe endpoints claros para `ndnm-brazil` solicitar execução (`POST /graphs/run`), gerenciamento de workspaces (`POST /nexus/save`, `GET /nexus/load/{name}`), obter a lista detalhada de nodes (`GET /nodes/registry`), e potencialmente endpoints para interagir com `behaviors` dinâmicos (ex: `POST /nodes/{node_id}/refresh_dynamic_section/{section_name}`).
    * **Feedback Contínuo:** Envia atualizações granulares de estado via `ndnm-brazil` para o `ndnm-argos` (ex: "Node X processando", "Node Y aguardando dados", "Transferindo 5GB para Node Z", "Erro: Node W falhou ao ler arquivo").
4.  **Persistência de Workspaces (`nexus/`):**
    * **Gerenciamento:** Lida com a leitura e escrita dos arquivos JSON na pasta `nexus/`, garantindo a integridade dos dados salvos.

## 🚫 O Que NÃO Pertence Aqui

* **Comunicação Direta com Frontend:** Continua sendo **proibido**. `ndnm-hermes` é um serviço interno.
* **Interface do Usuário:** Zero UI.
* **Lógica Específica dos Nodes:** Apenas invoca os nodes e gerencia o fluxo de dados.
* **Observabilidade Detalhada:** Emite logs/eventos, mas a coleta e análise profunda é da `ndnm-exdoida`.

## 🛠️ Tecnologias Principais

* **Rust**
* **Axum:** Para sua API interna.
* **Tokio:** Para concorrência massiva.
* **Serde (com `serde_yaml`):** Para parsear os `config.yaml` complexos e JSONs de grafos.
* **Reqwest (ou similar):** Para comunicação HTTP com os nodes. (Pode precisar de suporte a streaming/multipart para dados grandes).
* **Walkdir (ou `std::fs`):** Para descoberta de nodes.
* **`ndnm-libs`:** **Fundamental** para as structs de configuração (`NodeConfig` V2, `Section`, `SlotTemplate`, `AppError`), `load_config` (atualizado), etc.
* **(Potencial) Gerenciamento de Processos:** Crates para iniciar/monitorar sub-processos (se `hermes` for responsável por iniciar os nodes).
* **(Potencial) Bibliotecas de Transferência de Dados:** Soluções para lidar com transferência eficiente de grandes volumes de dados (streaming, mmap, etc.).

**Em resumo (V2):** `ndnm-hermes` é o maestro inteligente que não só lê a partitura (grafo), mas também entende as capacidades e a dinâmica de cada músico (node), adaptando a performance (execução) em tempo real e garantindo que até os solos mais pesados (dados grandes) sejam entregues com maestria.
