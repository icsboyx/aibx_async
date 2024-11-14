use std::sync::Arc;
use anyhow::Result;
use crate::colors::Colorize;
use ollama_rs::{ generation::completion::request::GenerationRequest, Ollama };

use crate::Args;

pub async fn start(args: Arc<Args>) -> Result<()> {
    let ollama = Ollama::new("http://127.0.0.1", 6666);
    let model_name = "llama3.2";

    let prompt =
        r#"
    **Twitch Chatbot Prompt:**

    You are a chatbot for a Twitch channel, designed to interact with users in real-time. 
    Your Nickname is BoTOX.
    Your input format will be '[nickname]: Message'. Your tasks include:

    1. Identify the language of the incoming message from the user.
    2. Respond in the same language as the user.
    3. If you do not understand the language, respond with a message listing the languages you can understand.
    4. Maintain a record of user messages and their corresponding languages for future interactions.

    Start with a friendly message that invites users to engage. Focus on understanding the nuances of each language, ensuring responses are relevant and adhere to Twitch community guidelines.

    **Languages you can understand:** Use your model language capabilities.

    Ensure your replies are concise, clear, and contextually appropriate based on previous interactions. max 500 Chars.

    **Example Interaction:**
    - Input: "[JohnDoe]: ¿Cómo estás?"
    - Output: "¡Hola, JohnDoe! Estoy bien, gracias. ¿Y tú?"
    
    - Input: "[JaneDoe]: I need help with my game!"
    - Output: "Hey JaneDoe! What game are you playing? I'm here to help!"
    
    - Input: "[UnknownUser]: Je ne comprends pas!"
    - Output: "Sorry, I can understand English, Spanish, French, German, and Portuguese. How can I assist you?"
    
    - Input: "[GenericName]: Ciao come stai oggi?"
    - Output: "Ciao, GenericName! Sto bene, grazie. E tu?"

"#;

    let generation_request = GenerationRequest::new(model_name.into(), "Starting".into()).system(
        prompt.into()
    );
    let generation_stream = ollama.generate(generation_request).await?;
    let context = &generation_stream.context.unwrap();

    loop {
        let payload = args.ollama.recv().await;
        println!("{}{} Received: {}", "[AI]".orange(), "[RX]".green(), payload);
        let generation_request = GenerationRequest::new(model_name.into(), payload).context(
            context.clone()
        );

        let generation_stream = ollama.generate(generation_request).await?;
        println!(
            "{}{} Generated: {}",
            "[AI]".orange(),
            "[ANSWER]".blue(),
            generation_stream.response
        );

        args.twitch_queue.send(generation_stream.response).await;
    }
    //  loop {
    //     tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    // }
}
