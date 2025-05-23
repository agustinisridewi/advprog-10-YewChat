use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleDarkMode,
    ClearChat,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    color: String, // Ganti avatar dengan warna
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    dark_mode: bool,
}

impl Chat {
    // Fungsi untuk generate warna berdasarkan nama user
    fn get_user_color(name: &str) -> String {
        let colors = vec![
            "#EF4444", "#F97316", "#F59E0B", "#EAB308", 
            "#84CC16", "#22C55E", "#10B981", "#14B8A6",
            "#06B6D4", "#0EA5E9", "#3B82F6", "#6366F1",
            "#8B5CF6", "#A855F7", "#D946EF", "#EC4899"
        ];
        let index = name.chars().map(|c| c as usize).sum::<usize>() % colors.len();
        colors[index].to_string()
    }
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            dark_mode: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.clone(),
                                color: Self::get_user_color(u),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let value = input.value().trim().to_string();
                    if !value.is_empty() {
                        let message = WebSocketMessage {
                            message_type: MsgTypes::Message,
                            data: Some(value),
                            data_array: None,
                        };
                        if let Err(e) = self
                            .wss
                            .tx
                            .clone()
                            .try_send(serde_json::to_string(&message).unwrap())
                        {
                            log::debug!("error sending to channel: {:?}", e);
                        }
                        input.set_value("");
                    }
                };
                false
            }
            Msg::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                true
            }
            Msg::ClearChat => {
                self.messages.clear();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_dark = ctx.link().callback(|_| Msg::ToggleDarkMode);
        let clear_chat = ctx.link().callback(|_| Msg::ClearChat);
        
        // Theme classes
        let bg_primary = if self.dark_mode { "bg-gray-900" } else { "bg-white" };
        let bg_secondary = if self.dark_mode { "bg-gray-800" } else { "bg-gray-50" };
        let bg_tertiary = if self.dark_mode { "bg-gray-700" } else { "bg-white" };
        let text_primary = if self.dark_mode { "text-white" } else { "text-gray-900" };
        let text_secondary = if self.dark_mode { "text-gray-300" } else { "text-gray-600" };
        let border_color = if self.dark_mode { "border-gray-700" } else { "border-gray-200" };

        html! {
            <div class={format!("flex h-screen {}", bg_primary)}>
                // Sidebar
                <div class={format!("flex-none w-80 {} border-r {}", bg_secondary, border_color)}>
                    // Header sidebar
                    <div class={format!("flex items-center justify-between p-4 border-b {}", border_color)}>
                        <h2 class={format!("text-lg font-semibold {}", text_primary)}>
                            {"Online Users"}
                        </h2>
                        <span class={format!("bg-green-500 text-white text-xs px-2 py-1 rounded-full")}>
                            {self.users.len()}
                        </span>
                    </div>
                    
                    // Users list
                    <div class="overflow-y-auto h-full pb-20">
                        {
                            if self.users.is_empty() {
                                html! {
                                    <div class={format!("flex items-center justify-center h-32 {}", text_secondary)}>
                                        <div class="text-center">
                                            <div class="text-2xl mb-2">{"😴"}</div>
                                            <div class="text-sm">{"No users online"}</div>
                                        </div>
                                    </div>
                                }
                            } else {
                                self.users.iter().map(|u| {
                                    html!{
                                        <div class={format!("flex items-center p-3 m-3 {} rounded-lg shadow-sm hover:shadow-md transition-shadow", bg_tertiary)}>
                                            <div 
                                                class="w-10 h-10 rounded-full flex items-center justify-center text-white font-bold text-sm mr-3"
                                                style={format!("background-color: {}", u.color)}
                                            >
                                                {u.name.chars().next().unwrap_or('?').to_uppercase()}
                                            </div>
                                            <div class="flex-1">
                                                <div class={format!("font-medium {}", text_primary)}>
                                                    {u.name.clone()}
                                                </div>
                                                <div class={format!("text-xs {}", text_secondary)}>
                                                    {"🟢 Online"}
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        }
                    </div>
                </div>

                // Main chat area
                <div class="flex-1 flex flex-col">
                    // Chat header
                    <div class={format!("flex items-center justify-between p-4 border-b {} {}", border_color, bg_tertiary)}>
                        <div class="flex items-center">
                            <h1 class={format!("text-xl font-bold {}", text_primary)}>
                                {"💬 Chat Room"}
                            </h1>
                            <span class={format!("ml-3 text-sm {} bg-blue-100 dark:bg-blue-900 px-2 py-1 rounded", text_secondary)}>
                                {format!("{} messages", self.messages.len())}
                            </span>
                        </div>
                        
                        <div class="flex items-center space-x-2">
                            // Clear chat button
                            <button 
                                onclick={clear_chat}
                                class="p-2 rounded-lg bg-red-500 hover:bg-red-600 text-white transition-colors"
                                title="Clear Chat"
                            >
                                {"🗑️"}
                            </button>
                            
                            // Dark mode toggle
                            <button 
                                onclick={toggle_dark}
                                class={format!("p-2 rounded-lg {} hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", text_primary)}
                                title="Toggle Dark Mode"
                            >
                                {if self.dark_mode { "☀️" } else { "🌙" }}
                            </button>
                        </div>
                    </div>

                    // Messages area
                    <div class={format!("flex-1 overflow-y-auto p-4 {}", bg_primary)}>
                        {
                            if self.messages.is_empty() {
                                html! {
                                    <div class={format!("flex items-center justify-center h-full {}", text_secondary)}>
                                        <div class="text-center">
                                            <div class="text-4xl mb-4">{"💭"}</div>
                                            <div class="text-lg">{"No messages yet"}</div>
                                            <div class="text-sm mt-2">{"Start a conversation!"}</div>
                                        </div>
                                    </div>
                                }
                            } else {
                                self.messages.iter().map(|m| {
                                    let user = self.users.iter().find(|u| u.name == m.from);
                                    let user_color = user.map(|u| u.color.clone()).unwrap_or_else(|| Self::get_user_color(&m.from));
                                    
                                    html!{
                                        <div class="mb-4 max-w-3xl">
                                            <div class={format!("flex items-start p-4 {} rounded-lg shadow-sm", bg_tertiary)}>
                                                <div 
                                                    class="w-8 h-8 rounded-full flex items-center justify-center text-white font-bold text-xs mr-3 flex-shrink-0"
                                                    style={format!("background-color: {}", user_color)}
                                                >
                                                    {m.from.chars().next().unwrap_or('?').to_uppercase()}
                                                </div>
                                                <div class="flex-1 min-w-0">
                                                    <div class={format!("font-medium text-sm mb-1 {}", text_primary)}>
                                                        {m.from.clone()}
                                                    </div>
                                                    <div class={format!("text-sm {}", text_primary)}>
                                                        if m.message.ends_with(".gif") || m.message.ends_with(".jpg") || m.message.ends_with(".png") {
                                                            <img class="mt-2 max-w-xs rounded-lg" src={m.message.clone()} alt="Image"/>
                                                        } else {
                                                            {m.message.clone()}
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        }
                    </div>

                    // Input area
                    <div class={format!("p-4 border-t {} {}", border_color, bg_tertiary)}>
                        <div class="flex items-end space-x-3">
                            <div class="flex-1">
                                <input 
                                    ref={self.chat_input.clone()} 
                                    type="text" 
                                    placeholder="Type your message..." 
                                    class={format!("w-full px-4 py-3 {} {} border {} rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none transition-colors", bg_primary, text_primary, border_color)}
                                    onkeypress={ctx.link().callback(|e: KeyboardEvent| {
                                        if e.key() == "Enter" && !e.shift_key() {
                                            e.prevent_default();
                                            Msg::SubmitMessage
                                        } else {
                                            return Msg::HandleMsg("".to_string()); // Dummy message
                                        }
                                    })}
                                />
                            </div>
                            <button 
                                onclick={submit}
                                class="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors flex items-center space-x-2"
                            >
                                <span>{"Send"}</span>
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}