# ndnm-exdoida: A Observadora Independente do NDNM

## 🕵️‍♀️ A Testemunha Silenciosa (e Anotadora Compulsiva)

`ndnm-exdoida` é o módulo dedicado à **observabilidade** do ecossistema NDNM. Sua missão é coletar, processar e armazenar (ou expor) logs, métricas e/ou traces gerados pelos outros componentes (`ndnm-hermes`, `ndnm-brazil`, `nodes/*`), funcionando de forma **altamente desacoplada e resiliente**.

O nome reflete sua natureza: ela "observa" tudo o que os outros fazem, anota os detalhes importantes (e os podres!), e continua funcionando mesmo que o "relacionamento" principal (a execução do grafo) tenha problemas. Ela é a nossa "caixa-preta", essencial para depuração e monitoramento.

## 🎯 Principais Responsabilidades

1.  **Coleta de Dados de Observabilidade:**
    * **Recepção de Logs:** Configurada para receber logs emitidos pelos outros módulos. Isso pode ser feito de várias formas desacopladas:
        * Ouvindo mensagens enviadas via UDP (fire-and-forget).
        * Assinando um tópico em um message broker (se usarmos um no futuro).
        * Lendo arquivos de log em um diretório compartilhado (menos ideal, mas possível).
        * Recebendo requisições HTTP diretas (se os outros módulos forem configurados para "empurrar" os logs).
    * **Coleta de Métricas (Futuro):** Pode ser estendida para receber métricas (ex: tempo de execução de nodes, uso de CPU/memória) via protocolos como Prometheus/StatsD.
    * **Coleta de Traces (Futuro):** Pode integrar-se a sistemas de tracing distribuído (ex: OpenTelemetry) para rastrear o fluxo de uma requisição através dos diferentes módulos.
2.  **Processamento e Armazenamento:**
    * **Parsing:** Interpretar os logs/métricas recebidos (ex: JSON, formato de texto).
    * **Enriquecimento:** Adicionar metadados úteis (timestamps, origem do log, etc.).
    * **Armazenamento:** Persistir os dados em um local apropriado (ex: arquivos de log rotacionados, banco de dados de logs como Loki/Elasticsearch, banco de dados de séries temporais como InfluxDB/Prometheus).
3.  **Exposição de Dados (Opcional):**
    * **API de Consulta:** Pode expor uma API HTTP simples para consultar logs recentes ou status.
    * **Dashboard:** Pode incluir uma interface web básica (ou integrar com ferramentas como Grafana) para visualização dos dados coletados.

## 🚫 O Que NÃO Pertence Aqui

* **Orquestração de Fluxo:** `ndnm-exdoida` **não** participa da lógica de execução do grafo. Ela apenas observa.
* **Comunicação Ativa com Outros Módulos (Idealmente):** Para manter o desacoplamento, `exdoida` não deve *iniciar* comunicação com `hermes` ou `brazil` para obter informações. Ela deve ser passiva, recebendo os dados que são enviados a ela.
* **Lógica de Negócio Principal:** Não contém nenhuma lógica relacionada à funcionalidade dos nodes ou à interface com o usuário.
* **Dependência Crítica:** O sistema principal (`hermes`, `brazil`, `nodes`) **não deve depender** do funcionamento da `exdoida` para operar. Se a `exdoida` cair, o resto continua funcionando (apenas sem a observabilidade detalhada).

## 💡 Estratégia de Desacoplamento

A chave para a resiliência da `ndnm-exdoida` é o **desacoplamento**. Os outros módulos devem enviar seus dados de observabilidade sem esperar confirmação ou resposta (ex: UDP, logar para stdout/stderr que é capturado externamente, etc.). Isso garante que uma falha na `exdoida` não trave o resto do sistema.

## 🛠️ Tecnologias Principais (Sugestões)

* **Rust**
* **Axum (Opcional):** Se for expor uma API HTTP para consulta.
* **Tokio:** Para a runtime assíncrona (ex: receber dados via UDP/TCP).
* **Serde:** Para parsear logs em formato JSON.
* **Tracing / Log:** Crates para instrumentação e formatação de logs.
* **Bibliotecas de Cliente (Futuro):** Clientes para bancos de dados de logs/métricas (Loki, InfluxDB, etc.).

**Em resumo:** `ndnm-exdoida` é a guardiã silenciosa da sanidade do sistema. Ela vê tudo, anota tudo, e nos ajuda a entender o caos quando as coisas (inevitavelmente) quebram. Sua independência é sua maior força.
