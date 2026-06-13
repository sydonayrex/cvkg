/**
 * Message data structure inside the AgentChat workspace.
 */
export interface AgentMessage {
  sender: "user" | "assistant";
  content: string;
}

/**
 * AgentChat component properties.
 */
export interface AgentChatProps {
  /** Selected assistant model. */
  model?: string;
  /** Active message list array history. */
  messages: AgentMessage[];
  /** Callback triggered when a new user prompt is submitted. */
  onSend?: (prompt: string) => void;
}

/**
 * Portable representation of the AgentChat panel workspace.
 */
export class AgentChat {
  private model: string;
  private messages: AgentMessage[];
  private onSend?: (prompt: string) => void;

  /**
   * Constructs a new AgentChat instance.
   */
  constructor(props: AgentChatProps) {
    this.model = props.model ?? "Gemini 3.5 Flash";
    this.messages = props.messages;
    this.onSend = props.onSend;
  }

  /**
   * Appends messages and redraws the chat window.
   */
  public addMessage(msg: AgentMessage, container: HTMLElement): void {
    this.messages.push(msg);
    this.render(container);
  }

  /**
   * Renders the chat panel workspace inside the container.
   */
  public render(container: HTMLElement): void {
    container.innerHTML = "";
    
    const panel = document.createElement("div");
    panel.style.display = "flex";
    panel.style.flexDirection = "column";
    panel.style.width = "100%";
    panel.style.height = "100%";
    panel.style.background = "#18181c";
    panel.style.border = "1px solid #333";
    panel.style.borderRadius = "8px";

    const header = document.createElement("div");
    header.style.padding = "12px";
    header.style.borderBottom = "1px solid #333";
    header.style.color = "#fff";
    header.style.fontWeight = "bold";
    header.style.fontSize = "14px";
    header.textContent = `Chat Workspace — Model: ${this.model}`;
    panel.appendChild(header);

    const messageList = document.createElement("div");
    messageList.style.flex = "1";
    messageList.style.overflowY = "auto";
    messageList.style.padding = "16px";
    messageList.style.display = "flex";
    messageList.style.flexDirection = "column";
    messageList.style.gap = "12px";

    this.messages.forEach(msg => {
      const bubble = document.createElement("div");
      bubble.style.padding = "10px 14px";
      bubble.style.borderRadius = "6px";
      bubble.style.maxWidth = "70%";
      bubble.style.fontSize = "13px";

      if (msg.sender === "user") {
        bubble.style.alignSelf = "flex-end";
        bubble.style.background = "#0080ff";
        bubble.style.color = "#fff";
      } else {
        bubble.style.alignSelf = "flex-start";
        bubble.style.background = "#282a36";
        bubble.style.color = "#f8f8f2";
        bubble.style.border = "1px solid #44475a";
      }
      bubble.textContent = msg.content;
      messageList.appendChild(bubble);
    });
    panel.appendChild(messageList);

    const inputBar = document.createElement("div");
    inputBar.style.padding = "12px";
    inputBar.style.borderTop = "1px solid #333";
    inputBar.style.display = "flex";
    inputBar.style.gap = "8px";

    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = "Ask assistant anything...";
    input.style.flex = "1";
    input.style.padding = "8px";
    input.style.borderRadius = "4px";

    const sendBtn = document.createElement("button");
    sendBtn.textContent = "Send";
    sendBtn.style.padding = "8px 16px";

    const sendPrompt = () => {
      const val = input.value.trim();
      if (val) {
        if (this.onSend) {
          this.onSend(val);
        }
        this.addMessage({ sender: "user", content: val }, container);
        input.value = "";
      }
    };

    sendBtn.addEventListener("click", sendPrompt);
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") sendPrompt();
    });

    inputBar.appendChild(input);
    inputBar.appendChild(sendBtn);
    panel.appendChild(inputBar);

    container.appendChild(panel);
  }
}
