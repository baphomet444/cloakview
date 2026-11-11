# Task 10 — Answers: Responses API streaming

## Decisiones que corrigen el plan (aplicar, no reinterpretar)

1. Endpoint: POST https://api.openai.com/v1/responses con "stream": true.
   El parser del plan era de Chat Completions y NO sirve.

2. Formato SSE real de Responses API. Eventos tipados en el campo "type":
   - response.output_text.delta -> Delta { text: v["delta"] }
   - response.completed         -> Done
   - response.failed            -> Error { message: v["response"]["error"]["message"] }
   - error                      -> Error { message: v["message"] }
   Rutas distintas: response.failed anida el error bajo "response".
   No existe el centinela "data: [DONE]".

3. AnswersEvent definitivo:
   enum AnswersEvent {
       Delta { text: String },
       Done,
       Error { message: String },
   }
   Derives: Debug, Clone, PartialEq, Serialize.
   serde(tag = "kind", rename_all = "camelCase"). Emitido como "answers".

   IMPORTANTE: #[serde(tag = "kind")] es internal tagging y NO soporta
   variantes newtype como Delta(String). Debe ser struct    variantes newtype como Delta(String). Debe ser struct    v message } en Task 9.

4. Buffer de líneas obligatorio. bytes_stream() no alinea chunks con líneas.
   Acumular en String, drenar solo líneas completas terminadas en \n,
   conservar el resto. Test explícito: alimentar una línea partida en dos.

5. stream_answer lanza la tarea y devuelve (Receiver<AnswersEvent>, JoinHandle).
   Usar tokio::sync::mpsc::channel(64) con try_send. Match obligatorio:
   - Full   -> descartar el delta (la UI se actualiza con el si   - Full   -> descartar el delta (la UI se actualiza con el si   ra
   No usar unbounded. No ignorar Closed: d   No usar unbounded. Noun
   stream HTTP muerto hasta completarse.

6. Contexto: presupuesto de TOKENS, no de turnos. Aproximar 4 chars/token,
   incluir Turn más recientes primero hasta ~2000 tokens. Van como
   user/assistant en el campo "input" de Responses API.

7. Authorization: Bearer <api_key> en header. Nunca en query string.

## Tests (escribir primero, en rojo)

- parse: delta real de Responses API
- parse: response.completed -> Done
- parse: response.failed -> Error con mensaje anidado
- parse: línea partida en dos chunks se parsea al completarse
- parse: ": keepalive" y línea vacía -> None
- parse: data con JSON roto -> None, sin panic
- serialize: AnswersEvent::Delta { t- serialize: AnswersEvent::Delta { t- serialize: AnswersEvent::Delta { t- seria## Cierre

cargo test, cargo build, clippy -D warnings. task-10-report.md.
Commit: feat: streaming answers via Responses API
No empieces la Task 11.
