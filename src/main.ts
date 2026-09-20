import { invoke } from "@tauri-apps/api/core";

type Draft = {
  draft: string;
  verify: string[];
};

type Question = {
  id: string;
  asked_at: string;
  asker: string;
  context: string;
  question: string;
  tags: string[];
  draft: Draft | null;
};

const form = document.querySelector<HTMLFormElement>("#question-form");
const questionList = document.querySelector<HTMLDivElement>("#question-list");
const questionCount = document.querySelector<HTMLSpanElement>("#question-count");
const errorMessage = document.querySelector<HTMLDivElement>("#error-message");
const formStatus = document.querySelector<HTMLParagraphElement>("#form-status");

function showError(message: string): void {
  if (!errorMessage) return;
  errorMessage.textContent = message;
  errorMessage.hidden = false;
}

function clearError(): void {
  if (!errorMessage) return;
  errorMessage.textContent = "";
  errorMessage.hidden = true;
}

function setFormStatus(message: string, tone: "success" | "error" = "success"): void {
  if (!formStatus) return;
  formStatus.textContent = message;
  formStatus.dataset.tone = tone;
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function createTag(tag: string): HTMLSpanElement {
  const element = document.createElement("span");
  element.className = "tag";
  element.textContent = tag;
  return element;
}

function renderQuestions(questions: Question[]): void {
  if (!questionList || !questionCount) return;
  questionCount.textContent = String(questions.length);
  questionList.replaceChildren();

  if (questions.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.innerHTML = "<strong>Your desk is clear.</strong><span>Log the first question when the next one arrives.</span>";
    questionList.append(empty);
    return;
  }

  questions.forEach((question) => {
    const card = document.createElement("article");
    card.className = "question-card";
    card.dataset.id = question.id;

    const meta = document.createElement("div");
    meta.className = "question-meta";
    const date = document.createElement("time");
    date.dateTime = question.asked_at;
    date.textContent = formatDate(question.asked_at);
    meta.append(date);
    if (question.asker) {
      const asker = document.createElement("span");
      asker.textContent = question.asker;
      meta.append(asker);
    }

    const title = document.createElement("h3");
    title.textContent = question.question;
    card.append(meta, title);

    if (question.context) {
      const context = document.createElement("p");
      context.className = "context-line";
      context.textContent = question.context;
      card.append(context);
    }

    if (question.tags.length > 0) {
      const tags = document.createElement("div");
      tags.className = "tag-list";
      question.tags.forEach((tag) => tags.append(createTag(tag)));
      card.append(tags);
    }

    const actions = document.createElement("div");
    actions.className = "card-actions";
    const draftButton = document.createElement("button");
    draftButton.className = "secondary-button";
    draftButton.type = "button";
    draftButton.dataset.action = "draft";
    draftButton.textContent = question.draft ? "Refresh draft" : "Draft response";
    actions.append(draftButton);

    const deleteButton = document.createElement("button");
    deleteButton.className = "text-button";
    deleteButton.type = "button";
    deleteButton.dataset.action = "delete";
    deleteButton.textContent = "Delete";
    actions.append(deleteButton);
    card.append(actions);

    if (question.draft) {
      const draftPanel = document.createElement("section");
      draftPanel.className = "draft-panel";
      const draftLabel = document.createElement("p");
      draftLabel.className = "draft-label";
      draftLabel.textContent = "AI draft - verify before use";
      const draftText = document.createElement("p");
      draftText.textContent = question.draft.draft;
      const verifyTitle = document.createElement("p");
      verifyTitle.className = "verify-title";
      verifyTitle.textContent = "Verify these";
      const verifyList = document.createElement("ul");
      question.draft.verify.forEach((item) => {
        const listItem = document.createElement("li");
        listItem.textContent = item;
        verifyList.append(listItem);
      });
      draftPanel.append(draftLabel, draftText, verifyTitle, verifyList);
      card.append(draftPanel);
    }

    questionList.append(card);
  });
}

async function loadQuestions(): Promise<void> {
  try {
    clearError();
    const questions = await invoke<Question[]>("list_questions");
    renderQuestions(questions);
  } catch (error) {
    showError(String(error));
  }
}

form?.addEventListener("submit", async (event) => {
  event.preventDefault();
  const formData = new FormData(form);
  const question = String(formData.get("question") ?? "").trim();
  const tags = String(formData.get("tags") ?? "")
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);

  try {
    clearError();
    setFormStatus("Saving...");
    await invoke<Question>("save_question", {
      asker: String(formData.get("asker") ?? ""),
      context: String(formData.get("context") ?? ""),
      question,
      tags,
    });
    form.reset();
    setFormStatus("Question saved.");
    await loadQuestions();
  } catch (error) {
    setFormStatus("Could not save question.", "error");
    showError(String(error));
  }
});

questionList?.addEventListener("click", async (event) => {
  const target = event.target;
  if (!(target instanceof HTMLButtonElement)) return;
  const card = target.closest<HTMLElement>(".question-card");
  const id = card?.dataset.id;
  const action = target.dataset.action;
  if (!id || !action) return;

  try {
    clearError();
    target.disabled = true;
    if (action === "delete") {
      if (!window.confirm("Delete this question and its draft?")) return;
      await invoke("delete_question", { id });
    } else if (action === "draft") {
      target.textContent = "Drafting...";
      await invoke<Question>("draft_answer", { id });
    }
    await loadQuestions();
  } catch (error) {
    showError(String(error));
    target.disabled = false;
  }
});

void loadQuestions();
