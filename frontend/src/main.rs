use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use web_sys::{InputEvent, RequestCredentials};
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize, Serialize)]
struct Transaction {
    pub id: Option<i32>,
    pub date: String,
    pub description: String,
    pub category: String,
    pub amount: i64,
}

const API_BASE_URL: &str = "http://localhost:5000";

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct AppSettings {
    currency_code: String,
    currency_symbol: String,
}

fn default_settings() -> AppSettings {
    AppSettings {
        currency_code: "PHP".to_string(),
        currency_symbol: "₱".to_string(),
    }
}

fn load_settings() -> AppSettings {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(raw)) = storage.get_item("settings") {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&raw) {
                    return settings;
                }
            }
        }
    }
    default_settings()
}

fn save_settings(settings: &AppSettings) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(raw) = serde_json::to_string(settings) {
                let _ = storage.set_item("settings", &raw);
            }
        }
    }
}

fn currency_symbol_for(code: &str) -> &'static str {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "₱",
    }
}

#[derive(Clone, Copy, PartialEq)]
enum AuthStatus {
    Checking,
    Authenticated,
    Unauthenticated,
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Dashboard,
    Budget,
    Income,
    Expense,
    Savings,
    Summary,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
enum StatIcon {
    UpRight,
    CreditCard,
    Wallet,
}

#[derive(Properties, PartialEq)]
struct LayoutProps {
    children: Children,
    active_page: Page,
    on_select: Callback<Page>,
}

#[function_component(Layout)]
fn layout(props: &LayoutProps) -> Html {
    html! {
        <div class="flex h-screen bg-background">
            <div class="hidden md:flex">
                <Sidebar active_page={props.active_page} on_select={props.on_select.clone()} />
            </div>

            <div class="flex-1 flex flex-col overflow-hidden">
                <Header />
                <main class="flex-1 overflow-y-auto">
                    { for props.children.iter() }
                </main>
            </div>
        </div>
    }
}

#[function_component(Header)]
fn header() -> Html {
    let show_notifications = use_state(|| false);
    let toggle_notifications = {
        let show_notifications = show_notifications.clone();
        Callback::from(move |_| show_notifications.set(!*show_notifications))
    };

    let notifications = vec![
        (
            "Saving Milestone!",
            "You've reached 30% of your goal.",
            "Just now",
        ),
        (
            "Goal Accomplished! 🏆",
            "You've reached 100% of your goal.",
            "1h ago",
        ),
    ];

    html! {
        <header class="bg-[#D8E1E8] border-b border-border h-16 flex items-center justify-between px-6">
            <div class="flex-1"></div>
            <div class="relative flex items-center gap-4">
                <button class="p-2 hover:bg-secondary rounded-full transition-colors relative" aria-label="Notifications" onclick={toggle_notifications}>
                    { icon_bell() }
                    <span class="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full"></span>
                </button>
                {
                    if *show_notifications {
                        html! {
                            <div class="absolute right-0 top-12 w-80 bg-white border border-border rounded-xl shadow-lg overflow-hidden z-50">
                                <div class="px-4 py-3 border-b border-border">
                                    <h4 class="text-sm font-bold text-[#173E63]">{"Notifications"}</h4>
                                </div>
                                <div class="divide-y divide-border">
                                    { for notifications.iter().map(|(title, message, time)| html! {
                                        <div class="px-4 py-3 hover:bg-slate-50">
                                            <div class="flex items-center justify-between">
                                                <p class="text-sm font-bold text-[#173E63]">{ *title }</p>
                                                <span class="text-[10px] text-slate-400 font-bold uppercase tracking-tighter">{ *time }</span>
                                            </div>
                                            <p class="text-xs text-slate-500 mt-1">{ *message }</p>
                                        </div>
                                    }) }
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        </header>
    }
}

struct NavItem {
    label: &'static str,
    page: Page,
    icon: fn() -> Html,
}

#[derive(Properties, PartialEq)]
struct SidebarProps {
    active_page: Page,
    on_select: Callback<Page>,
}

#[function_component(Sidebar)]
fn sidebar(props: &SidebarProps) -> Html {
    let nav_items = vec![
        NavItem {
            label: "Dashboard",
            page: Page::Dashboard,
            icon: icon_layout_grid,
        },
        NavItem {
            label: "Budget",
            page: Page::Budget,
            icon: icon_wallet,
        },
        NavItem {
            label: "Income Tracker",
            page: Page::Income,
            icon: icon_trending_up,
        },
        NavItem {
            label: "Expense Tracker",
            page: Page::Expense,
            icon: icon_credit_card,
        },
        NavItem {
            label: "Saving Goal",
            page: Page::Savings,
            icon: icon_target,
        },
        NavItem {
            label: "Summary Report",
            page: Page::Summary,
            icon: icon_bar_chart,
        },
        NavItem {
            label: "Settings",
            page: Page::Settings,
            icon: icon_settings,
        },
    ];

    let on_logout = Callback::from(move |_| {
        spawn_local(async move {
            let url = format!("{}/api/auth/logout", API_BASE_URL);
            let _ = Request::post(&url)
                .credentials(RequestCredentials::Include)
                .send()
                .await;
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.remove_item("access_token");
                }
                let _ = window.location().reload();
            }
        });
    });

    html! {
        <div class="w-[220px] h-screen bg-[#D8E1E8] p-4 flex flex-col">
            <div class="flex items-center gap-3 px-2 mb-8">
                <div class="w-12 h-12 bg-[#173E63] rounded-full flex items-center justify-center">
                    <img src="PondoBro.png" alt="Logo" class="w-full h-full object-cover rounded-full" />
                </div>
                <span class="text-[#173E63] text-2xl font-black tracking-tight">{"PondoBro"}</span>
            </div>

            <div class="flex-1 bg-[#173E63] rounded-[24px] flex flex-col py-6 px-3 shadow-lg">
                <nav class="flex-1 space-y-2">
                    { for nav_items.iter().map(|item| {
                        let is_active = item.page == props.active_page;
                        let class_name = if is_active {
                            "flex items-center gap-3 px-4 py-3 rounded-xl transition-all text-[13px] font-medium bg-[#B2CBDE] text-[#173E63] w-full"
                        } else {
                            "flex items-center gap-3 px-4 py-3 rounded-xl transition-all text-[13px] font-medium text-slate-300 hover:bg-white/5 hover:text-white w-full"
                        };
                        let on_select = props.on_select.clone();
                        let page = item.page;

                        html! {
                            <button type="button" class={class_name} onclick={Callback::from(move |_| on_select.emit(page))}>
                                <span class="shrink-0">{ (item.icon)() }</span>
                                <span class="truncate whitespace-nowrap text-left">{ item.label }</span>
                            </button>
                        }
                    }) }
                </nav>

                <div class="mt-auto pt-4">
                    <button onclick={on_logout} class="flex items-center gap-3 w-full px-4 py-3 rounded-xl hover:bg-white/10 transition-colors text-[13px] font-medium text-slate-300">
                        { icon_log_out() }
                        <span>{"Log Out"}</span>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[function_component(DashboardPage)]
fn dashboard_page() -> Html {
    let transactions = use_state(|| Vec::<Transaction>::new());
    let loading = use_state(|| true);
    let show_add = use_state(|| false);

    let settings = use_context::<UseStateHandle<AppSettings>>();
    let currency_symbol = settings
        .as_ref()
        .map(|s| s.currency_symbol.clone())
        .unwrap_or_else(|| "₱".to_string());

    let current_goal = load_saving_goal();

    let form_date = use_state(|| "".to_string());
    let form_description = use_state(|| "".to_string());
    let form_category = use_state(|| "".to_string());
    let form_amount = use_state(|| "".to_string());
    let form_error = use_state(|| None::<String>);
    let form_success = use_state(|| None::<String>);
    let saving = use_state(|| false);

    let budgets = use_state(load_budgets);

    // fetch transactions and dashboard summary
    let total_income = use_state(|| 0i64);
    let total_expenses = use_state(|| 0i64);
    let balance = use_state(|| 0i64);

    {
        let transactions = transactions.clone();
        let loading = loading.clone();
        let total_income = total_income.clone();
        let total_expenses = total_expenses.clone();
        let balance = balance.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let url = format!("{}/api/transactions", API_BASE_URL);
                    {
                        let mut req = Request::get(&url).credentials(RequestCredentials::Include);
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                if let Ok(token_opt) = storage.get_item("access_token") {
                                    if let Some(token) = token_opt {
                                        req = req
                                            .header("Authorization", &format!("Bearer {}", token));
                                    }
                                }
                            }
                        }

                        if let Ok(resp) = req.send().await {
                            if resp.ok() {
                                if let Ok(list) = resp.json::<Vec<Transaction>>().await {
                                    transactions.set(list);
                                }
                            }
                        }
                    }

                    // fetch dashboard summary
                    let summary_url = format!("{}/api/dashboard/summary", API_BASE_URL);
                    {
                        let mut req2 =
                            Request::get(&summary_url).credentials(RequestCredentials::Include);
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                if let Ok(token_opt) = storage.get_item("access_token") {
                                    if let Some(token) = token_opt {
                                        req2 = req2
                                            .header("Authorization", &format!("Bearer {}", token));
                                    }
                                }
                            }
                        }

                        if let Ok(resp2) = req2.send().await {
                            if resp2.ok() {
                                if let Ok(json) = resp2.json::<serde_json::Value>().await {
                                    if let Some(v) =
                                        json.get("total_income").and_then(|x| x.as_i64())
                                    {
                                        total_income.set(v);
                                    }
                                    if let Some(v) =
                                        json.get("total_expenses").and_then(|x| x.as_i64())
                                    {
                                        total_expenses.set(v);
                                    }
                                    if let Some(v) = json.get("balance").and_then(|x| x.as_i64()) {
                                        balance.set(v);
                                    }
                                }
                            }
                        }
                    }

                    loading.set(false);
                });
                || ()
            },
            (),
        );
    }

    let on_toggle_add = {
        let show_add = show_add.clone();
        let form_error = form_error.clone();
        let form_success = form_success.clone();
        Callback::from(move |_| {
            show_add.set(!*show_add);
            form_error.set(None);
            form_success.set(None);
        })
    };

    let on_submit = {
        let form_date = form_date.clone();
        let form_description = form_description.clone();
        let form_category = form_category.clone();
        let form_amount = form_amount.clone();
        let transactions = transactions.clone();
        let show_add = show_add.clone();
        let total_income = total_income.clone();
        let total_expenses = total_expenses.clone();
        let balance = balance.clone();
        let form_error = form_error.clone();
        let form_success = form_success.clone();
        let saving = saving.clone();

        Callback::from(move |_| {
            let form_date = form_date.clone();
            let form_description = form_description.clone();
            let form_category = form_category.clone();
            let form_amount = form_amount.clone();
            let transactions = transactions.clone();
            let show_add = show_add.clone();
            let total_income = total_income.clone();
            let total_expenses = total_expenses.clone();
            let balance = balance.clone();
            let form_error = form_error.clone();
            let form_success = form_success.clone();
            let saving = saving.clone();

            let date_val = form_date.trim().to_string();
            let desc_val = form_description.trim().to_string();
            let category_val = form_category.trim().to_string();
            let amount_val = form_amount.trim().to_string();

            if date_val.is_empty()
                || desc_val.is_empty()
                || category_val.is_empty()
                || amount_val.is_empty()
            {
                form_error.set(Some("Please complete all fields.".to_string()));
                return;
            }

            let amount = amount_val.parse::<i64>().unwrap_or(0);
            if amount == 0 {
                form_error.set(Some("Amount must be a non-zero number.".to_string()));
                return;
            }

            form_error.set(None);
            form_success.set(None);
            saving.set(true);

            spawn_local(async move {
                let url = format!("{}/api/transactions", API_BASE_URL);
                let payload = serde_json::json!({
                    "date": date_val.as_str(),
                    "description": desc_val.as_str(),
                    "category": category_val.as_str(),
                    "amount": amount
                });

                // build request (attach access token if available)
                let mut builder = Request::post(&url).credentials(RequestCredentials::Include);
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(token_opt) = storage.get_item("access_token") {
                            if let Some(token) = token_opt {
                                builder =
                                    builder.header("Authorization", &format!("Bearer {}", token));
                            }
                        }
                    }
                }

                let builder = match builder.json(&payload) {
                    Ok(b) => b,
                    Err(_) => return,
                };

                // send request
                let resp = match builder.send().await {
                    Ok(r) => r,
                    Err(_) => return,
                };

                if !resp.ok() {
                    form_error.set(Some("Could not save the transaction.".to_string()));
                    saving.set(false);
                    return;
                }

                if let Ok(created) = resp.json::<Transaction>().await {
                    let mut next = (*transactions).clone();
                    next.insert(0, created);
                    transactions.set(next);
                    // reset form
                    form_date.set("".to_string());
                    form_description.set("".to_string());
                    form_category.set("".to_string());
                    form_amount.set("".to_string());

                    // refresh dashboard summary
                    let summary_url = format!("{}/api/dashboard/summary", API_BASE_URL);
                    if let Ok(resp2) = Request::get(&summary_url)
                        .credentials(RequestCredentials::Include)
                        .send()
                        .await
                    {
                        if resp2.ok() {
                            if let Ok(json) = resp2.json::<serde_json::Value>().await {
                                if let Some(v) = json.get("total_income").and_then(|x| x.as_i64()) {
                                    total_income.set(v);
                                }
                                if let Some(v) = json.get("total_expenses").and_then(|x| x.as_i64())
                                {
                                    total_expenses.set(v);
                                }
                                if let Some(v) = json.get("balance").and_then(|x| x.as_i64()) {
                                    balance.set(v);
                                }
                            }
                        }
                    }

                    show_add.set(false);
                    form_success.set(Some("Transaction saved.".to_string()));
                    saving.set(false);
                } else {
                    form_error.set(Some("Could not read the saved transaction.".to_string()));
                    saving.set(false);
                }
            });
        })
    };


    let mut spent_by_category: HashMap<String, i64> = HashMap::new();
    for tx in (*transactions).iter() {
        if tx.amount < 0 {
            let spent = tx.amount.abs();
            *spent_by_category.entry(tx.category.clone()).or_insert(0) += spent;
        }
    }

    let total_budget: i64 = budgets.iter().map(|b| b.limit).sum();
    let budget_spent: i64 = budgets
        .iter()
        .map(|b| spent_by_category.get(&b.category).cloned().unwrap_or(0))
        .sum();
    let budget_remaining: i64 = total_budget - budget_spent;
    let overspent_count: usize = budgets
        .iter()
        .filter(|b| spent_by_category.get(&b.category).cloned().unwrap_or(0) > b.limit)
        .count();

    let goal_saved: i64 = current_goal.contributions.iter().map(|c| c.amount).sum();
    let goal_progress = if current_goal.target_amount > 0 {
        (goal_saved as f64 / current_goal.target_amount as f64).min(1.0)
    } else {
        0.0
    };

    html! {
        { page_shell(
            "Dashboard",
            html! {
                <button onclick={on_toggle_add} class="flex items-center gap-2 bg-primary text-primary-foreground px-4 py-2 rounded-xl font-bold text-sm hover:opacity-90 transition-all">
                    { icon_plus() }
                    { if *show_add { "Close" } else { "Add Transaction" } }
                </button>
            },
            html! {
                <>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <StatCard title="Total Income" amount={*total_income} icon={StatIcon::UpRight} currency_symbol={currency_symbol.clone()} />
                        <StatCard title="Total Expenses" amount={*total_expenses} icon={StatIcon::CreditCard} currency_symbol={currency_symbol.clone()} />
                        <StatCard title="Current Balance" amount={*balance} icon={StatIcon::Wallet} currency_symbol={currency_symbol.clone()} />
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        <div class="bg-card rounded-[10px] p-6 border border-border">
                            <div class="flex items-center justify-between mb-3">
                                <h3 class="font-bold text-foreground text-lg">{"Saving Goal"}</h3>
                                <span class="text-xs text-muted-foreground">{"Managed in Saving Goal tab"}</span>
                            </div>
                            { if current_goal.target_amount == 0 && current_goal.contributions.is_empty() && current_goal.title.trim().is_empty() {
                                html! { <p class="text-sm text-muted-foreground">{"No goal set yet."}</p> }
                            } else {
                                html! {
                                    <>
                                        <p class="text-sm text-muted-foreground">{ if current_goal.title.trim().is_empty() { "Saving Goal" } else { current_goal.title.as_str() } }</p>
                                        <div class="mt-3 flex items-center justify-between text-sm">
                                            <span class="text-muted-foreground">{ format!("Saved: {}", format_currency(goal_saved, &currency_symbol)) }</span>
                                            <span class="text-muted-foreground">{ if current_goal.target_amount > 0 { format!("Target: {}", format_currency(current_goal.target_amount, &currency_symbol)) } else { "Target: —".to_string() } }</span>
                                        </div>
                                        <div class="mt-2 h-2 w-full bg-secondary rounded-full overflow-hidden">
                                            <div class="h-full bg-primary" style={format!("width: {}%", (goal_progress * 100.0) as i32)}></div>
                                        </div>
                                        <p class="mt-2 text-xs text-muted-foreground">{ format!("{}% complete", (goal_progress * 100.0) as i32) }</p>
                                    </>
                                }
                            }}
                        </div>

                        <div class="bg-card rounded-[10px] p-6 border border-border">
                            <div class="flex items-center justify-between mb-3">
                                <h3 class="font-bold text-foreground text-lg">{"Budget Status"}</h3>
                                <span class="text-xs text-muted-foreground">{"Based on expenses"}</span>
                            </div>
                            { if budgets.is_empty() {
                                html! { <p class="text-sm text-muted-foreground">{"No budgets set yet."}</p> }
                            } else {
                                html! {
                                    <>
                                        <div class="flex items-center justify-between text-sm mb-3">
                                            <span class="text-muted-foreground">{"Remaining overall"}</span>
                                            <span class={if budget_remaining < 0 { "text-red-600" } else { "text-foreground" }}>
                                                { format_currency(budget_remaining.abs(), &currency_symbol) }
                                            </span>
                                        </div>
                                        { if overspent_count > 0 {
                                            html! { <p class="text-xs text-red-600 mb-3">{ format!("{} budget(s) over limit", overspent_count) }</p> }
                                        } else { html!{} } }
                                        <div class="space-y-2">
                                            { for budgets.iter().map(|b| {
                                                let spent = spent_by_category.get(&b.category).cloned().unwrap_or(0);
                                                let remaining = b.limit - spent;
                                                let percent = if b.limit > 0 { (spent as f64 / b.limit as f64 * 100.0).round() as i64 } else { 0 };
                                                html! {
                                                    <div class="flex flex-col gap-1 text-sm">
                                                        <div class="flex items-center justify-between">
                                                            <span class="text-foreground">{ b.category.clone() }</span>
                                                            <span class={if remaining < 0 { "text-red-600" } else { "text-muted-foreground" }}>
                                                                { format!("{}%", percent) }
                                                            </span>
                                                        </div>
                                                        <div class="h-2 w-full bg-secondary rounded-full overflow-hidden">
                                                            <div class="h-full bg-primary" style={format!("width: {}%", percent.min(100))}></div>
                                                        </div>
                                                        <div class="flex items-center justify-between text-xs text-muted-foreground">
                                                            <span>{ format!("Spent: {}", format_currency(spent, &currency_symbol)) }</span>
                                                            <span class={if remaining < 0 { "text-red-600" } else { "text-muted-foreground" }}>{ format!("Remaining: {}", format_currency(remaining.abs(), &currency_symbol)) }</span>
                                                        </div>
                                                    </div>
                                                }
                                            }) }
                                        </div>
                                    </>
                                }
                            }}
                        </div>
                    </div>

                    {
                        if *show_add {
                            html! {
                                <div class="bg-card rounded-[10px] p-6 mt-4 border border-border">
                                    <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                                        <input type="date" value={(*form_date).clone()} oninput={Callback::from(move |e: InputEvent| {
                                            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                form_date.set(input.value());
                                            }
                                        })} class="p-2 border rounded" />
                                        <input placeholder="Description" value={(*form_description).clone()} oninput={Callback::from(move |e: InputEvent| {
                                            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                form_description.set(input.value());
                                            }
                                        })} class="p-2 border rounded" />
                                        <input placeholder="Category" value={(*form_category).clone()} oninput={Callback::from(move |e: InputEvent| {
                                            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                form_category.set(input.value());
                                            }
                                        })} class="p-2 border rounded" />
                                        <div class="flex gap-2">
                                            <input placeholder={format!("Amount ({})", currency_symbol)} value={(*form_amount).clone()} oninput={Callback::from(move |e: InputEvent| {
                                                if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                    form_amount.set(input.value());
                                                }
                                            })} class="p-2 border rounded flex-1" />
                                            <button onclick={on_submit} class="bg-accent text-white px-4 rounded" disabled={*saving}>{ if *saving { "Saving..." } else { "Save" } }</button>
                                        </div>
                                        {
                                            if let Some(msg) = &*form_error {
                                                html! { <p class="text-sm text-red-500">{ msg.clone() }</p> }
                                            } else if let Some(msg) = &*form_success {
                                                html! { <p class="text-sm text-green-600">{ msg.clone() }</p> }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </div>
                                </div>
                            }
                        } else { html!{} }
                    }

                    <div class="bg-card rounded-[10px] shadow-sm border border-border overflow-hidden mt-4">
                        <div class="p-6 flex justify-between items-center border-b border-border">
                            <h3 class="font-bold text-foreground text-lg">{"Recent Transactions"}</h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-muted/50 text-muted-foreground text-[10px] uppercase tracking-widest">
                                        <th class="px-8 py-4 font-bold">{"Date"}</th>
                                        <th class="px-8 py-4 font-bold">{"Description"}</th>
                                        <th class="px-8 py-4 font-bold">{"Category"}</th>
                                        <th class="px-8 py-4 font-bold text-right">{"Amount"}</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-border">
                                    { for (*transactions).iter().enumerate().map(|(idx, tx)| {
                                        let amount_class = "px-8 py-4 text-right font-semibold text-foreground";
                                        let amount_label = if tx.amount > 0 {
                                            format!("+ {}", format_currency(tx.amount, &currency_symbol))
                                        } else {
                                            format_currency(tx.amount, &currency_symbol)
                                        };

                                        html! {
                                            <tr key={idx} class="text-sm hover:bg-muted/30 transition-colors">
                                                <td class="px-8 py-4 text-muted-foreground">{ &tx.date }</td>
                                                <td class="px-8 py-4 text-foreground">{ &tx.description }</td>
                                                <td class="px-8 py-4">
                                                    <span class="bg-secondary text-secondary-foreground px-3 py-1 rounded-full text-[10px] font-bold">{ &tx.category }</span>
                                                </td>
                                                <td class={amount_class}>{ amount_label }</td>
                                            </tr>
                                        }
                                    }) }
                                </tbody>
                            </table>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

fn page_shell(title: &'static str, actions: Html, children: Html) -> Html {
    html! {
        <div class="p-6 max-w-7xl mx-auto">
            <div class="flex items-center justify-between pb-4 border-b border-border">
                <h1 class="text-2xl font-bold text-foreground">{ title }</h1>
                { actions }
            </div>
            <div class="pt-5 space-y-6">
                { children }
            </div>
        </div>
    }
}

#[function_component(BudgetPage)]
fn budget_page() -> Html {
    let settings = use_context::<UseStateHandle<AppSettings>>();
    let currency_symbol = settings
        .as_ref()
        .map(|s| s.currency_symbol.clone())
        .unwrap_or_else(|| "₱".to_string());

    let category_totals = use_state(|| Vec::<(String, i64)>::new());
    let total_spent = use_state(|| 0i64);
    let loading = use_state(|| true);

    let budgets = use_state(load_budgets);
    let budget_category = use_state(|| "".to_string());
    let budget_limit = use_state(|| "".to_string());
    let budget_error = use_state(|| None::<String>);

    {
        let category_totals = category_totals.clone();
        let total_spent = total_spent.clone();
        let loading = loading.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let url = format!("{}/api/transactions", API_BASE_URL);
                    let mut req = Request::get(&url).credentials(RequestCredentials::Include);
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            if let Ok(token_opt) = storage.get_item("access_token") {
                                if let Some(token) = token_opt {
                                    req = req.header("Authorization", &format!("Bearer {}", token));
                                }
                            }
                        }
                    }

                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(list) = resp.json::<Vec<Transaction>>().await {
                                let mut totals: HashMap<String, i64> = HashMap::new();
                                let mut spent = 0i64;
                                for tx in list.iter() {
                                    if tx.amount < 0 {
                                        let amt = tx.amount.abs();
                                        spent += amt;
                                        *totals.entry(tx.category.clone()).or_insert(0) += amt;
                                    }
                                }
                                let mut totals_vec: Vec<(String, i64)> =
                                    totals.into_iter().collect();
                                totals_vec.sort_by(|a, b| b.1.cmp(&a.1));
                                category_totals.set(totals_vec);
                                total_spent.set(spent);
                            }
                        }
                    }

                    loading.set(false);
                });
                || ()
            },
            (),
        );
    }

    let on_add_budget = {
        let budgets = budgets.clone();
        let budget_category = budget_category.clone();
        let budget_limit = budget_limit.clone();
        let budget_error = budget_error.clone();
        Callback::from(move |_| {
            let category = budget_category.trim().to_string();
            let limit = budget_limit.trim().parse::<i64>().unwrap_or(0);

            if category.is_empty() || limit <= 0 {
                budget_error.set(Some("Enter a category and a positive limit.".to_string()));
                return;
            }

            let mut next = (*budgets).clone();
            if let Some(existing) = next
                .iter_mut()
                .find(|b| b.category.eq_ignore_ascii_case(&category))
            {
                existing.limit = limit;
            } else {
                next.push(BudgetItem { category, limit });
            }

            save_budgets(&next);
            budgets.set(next);
            budget_category.set("".to_string());
            budget_limit.set("".to_string());
            budget_error.set(None);
        })
    };

    let mut spent_by_category: HashMap<String, i64> = HashMap::new();
    for (cat, amt) in (*category_totals).iter() {
        spent_by_category.insert(cat.clone(), *amt);
    }
    html! {
        { page_shell(
            "Budget Overview",
            html! {},
            html! {
                <>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div class="bg-card rounded-[10px] p-6 border border-border">
                            <p class="text-sm text-muted-foreground mb-2">{"Total Expenses"}</p>
                            <h3 class="text-2xl font-bold text-foreground">{ format_currency(*total_spent, &currency_symbol) }</h3>
                            <p class="text-xs text-muted-foreground mt-2">{"Sum of all expense transactions"}</p>
                        </div>

                        <div class="bg-card rounded-[10px] p-6 border border-border">
                            <p class="text-sm text-muted-foreground mb-2">{"Top Categories"}</p>
                            <div class="space-y-2">
                                { if *loading {
                                    html! { <p class="text-sm text-muted-foreground">{"Loading..."}</p> }
                                } else if category_totals.is_empty() {
                                    html! { <p class="text-sm text-muted-foreground">{"No expense transactions yet."}</p> }
                                } else {
                                    html! {
                                        <ul class="space-y-1">
                                            { for category_totals.iter().take(5).map(|(cat, amt)| html! {
                                                <li class="flex items-center justify-between text-sm">
                                                    <span class="text-foreground">{ cat.clone() }</span>
                                                    <span class="font-semibold">{ format_currency(*amt, &currency_symbol) }</span>
                                                </li>
                                            }) }
                                        </ul>
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    <div class="bg-card rounded-[10px] p-6 border border-border">
                        <div class="flex items-center justify-between mb-4">
                            <h3 class="font-bold text-foreground text-lg">{"Category Budgets"}</h3>
                            <span class="text-xs text-muted-foreground">{"Set monthly limits"}</span>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-4">
                            <input placeholder="Category" value={(*budget_category).clone()} oninput={Callback::from({
                                let budget_category = budget_category.clone();
                                move |e: InputEvent| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        budget_category.set(input.value());
                                    }
                                }
                            })} class="p-2 border rounded" />
                            <input placeholder={format!("Limit ({})", currency_symbol)} value={(*budget_limit).clone()} oninput={Callback::from({
                                let budget_limit = budget_limit.clone();
                                move |e: InputEvent| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        budget_limit.set(input.value());
                                    }
                                }
                            })} class="p-2 border rounded" />
                            <button onclick={on_add_budget} class="bg-primary text-primary-foreground px-4 rounded">{"Save Budget"}</button>
                        </div>
                        {
                            if let Some(msg) = &*budget_error {
                                html! { <p class="text-sm text-red-500 mb-3">{ msg.clone() }</p> }
                            } else { html!{} }
                        }
                        <div class="space-y-2">
                            { if budgets.is_empty() {
                                html! { <p class="text-sm text-muted-foreground">{"No budgets yet."}</p> }
                            } else {
                                html! {
                                    <div class="space-y-2">
                                        { for budgets.iter().map(|b| {
                                            let spent = spent_by_category.get(&b.category).cloned().unwrap_or(0);
                                            let remaining = (b.limit - spent).max(0);
                                            let percent = if b.limit > 0 { (spent as f64 / b.limit as f64 * 100.0).round() as i64 } else { 0 };
                                            html! {
                                                <div class="flex flex-col gap-1 p-3 border rounded">
                                                    <div class="flex items-center justify-between">
                                                        <span class="font-semibold text-foreground">{ b.category.clone() }</span>
                                                        <span class="text-sm text-muted-foreground">{ format!("{}% used", percent) }</span>
                                                    </div>
                                                    <div class="flex items-center justify-between text-sm">
                                                        <span class="text-muted-foreground">{ format!("Spent: {}", format_currency(spent, &currency_symbol)) }</span>
                                                        <span class="text-muted-foreground">{ format!("Remaining: {}", format_currency(remaining, &currency_symbol)) }</span>
                                                    </div>
                                                </div>
                                            }
                                        }) }
                                    </div>
                                }
                            }}
                        </div>
                    </div>

                    <div>
                        <h2 class="text-lg font-bold text-foreground mb-3">{"Expense Breakdown"}</h2>
                        <div class="bg-card rounded-[10px] border border-border overflow-hidden">
                            <div class="p-6">
                                { if category_totals.is_empty() {
                                    html! { <p class="text-sm text-muted-foreground">{"Add expense transactions from the Dashboard to see breakdowns."}</p> }
                                } else {
                                    html! {
                                        <div class="space-y-2">
                                            { for category_totals.iter().map(|(cat, amt)| html! {
                                                <div class="flex items-center justify-between text-sm">
                                                    <span class="text-muted-foreground">{ cat.clone() }</span>
                                                    <span class="font-semibold text-foreground">{ format_currency(*amt, &currency_symbol) }</span>
                                                </div>
                                            }) }
                                        </div>
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

#[function_component(IncomePage)]
fn income_page() -> Html {
    let settings = use_context::<UseStateHandle<AppSettings>>();
    let currency_symbol = settings
        .as_ref()
        .map(|s| s.currency_symbol.clone())
        .unwrap_or_else(|| "₱".to_string());

    let incomes = use_state(|| Vec::<Transaction>::new());
    let loading = use_state(|| true);

    let form_date = use_state(|| "".to_string());
    let form_amount = use_state(|| "".to_string());
    let form_category = use_state(|| "Salary".to_string());
    let form_description = use_state(|| "".to_string());
    let form_error = use_state(|| None::<String>);
    let saving = use_state(|| false);

    {
        let incomes = incomes.clone();
        let loading = loading.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let url = format!("{}/api/transactions", API_BASE_URL);
                    let mut req = Request::get(&url).credentials(RequestCredentials::Include);
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            if let Ok(token_opt) = storage.get_item("access_token") {
                                if let Some(token) = token_opt {
                                    req = req.header("Authorization", &format!("Bearer {}", token));
                                }
                            }
                        }
                    }

                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(list) = resp.json::<Vec<Transaction>>().await {
                                let filtered = list
                                    .into_iter()
                                    .filter(|t| t.amount > 0)
                                    .collect::<Vec<_>>();
                                incomes.set(filtered);
                            }
                        }
                    }
                    loading.set(false);
                });
                || ()
            },
            (),
        );
    }

    let total_balance: i64 = incomes.iter().map(|item| item.amount).sum();

    let on_add = {
        let incomes = incomes.clone();
        let form_date = form_date.clone();
        let form_amount = form_amount.clone();
        let form_category = form_category.clone();
        let form_description = form_description.clone();
        let form_error = form_error.clone();
        let saving = saving.clone();
        Callback::from(move |_| {
            let date_val = form_date.trim().to_string();
            let desc_val = form_description.trim().to_string();
            let cat_val = form_category.trim().to_string();
            let amt_val = form_amount.trim().to_string();

            if date_val.is_empty()
                || desc_val.is_empty()
                || cat_val.is_empty()
                || amt_val.is_empty()
            {
                form_error.set(Some("Please complete all fields.".to_string()));
                return;
            }

            let parsed = amt_val.parse::<i64>().unwrap_or(0);
            if parsed <= 0 {
                form_error.set(Some("Amount must be a positive number.".to_string()));
                return;
            }

            form_error.set(None);
            saving.set(true);

            let incomes = incomes.clone();
            let form_date = form_date.clone();
            let form_amount = form_amount.clone();
            let form_category = form_category.clone();
            let form_description = form_description.clone();
            let saving = saving.clone();
            spawn_local(async move {
                let url = format!("{}/api/transactions", API_BASE_URL);
                let payload = serde_json::json!({
                    "date": date_val.as_str(),
                    "description": desc_val.as_str(),
                    "category": cat_val.as_str(),
                    "amount": parsed
                });

                let mut builder = Request::post(&url).credentials(RequestCredentials::Include);
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(token_opt) = storage.get_item("access_token") {
                            if let Some(token) = token_opt {
                                builder =
                                    builder.header("Authorization", &format!("Bearer {}", token));
                            }
                        }
                    }
                }

                let builder = match builder.json(&payload) {
                    Ok(b) => b,
                    Err(_) => return,
                };

                if let Ok(resp) = builder.send().await {
                    if resp.ok() {
                        if let Ok(created) = resp.json::<Transaction>().await {
                            let mut next = (*incomes).clone();
                            next.insert(0, created);
                            incomes.set(next);
                            form_date.set("".to_string());
                            form_amount.set("".to_string());
                            form_category.set("Salary".to_string());
                            form_description.set("".to_string());
                        }
                    }
                }
                saving.set(false);
            });
        })
    };

    let on_clear = {
        let form_amount = form_amount.clone();
        let form_description = form_description.clone();
        let form_date = form_date.clone();
        Callback::from(move |_| {
            form_date.set("".to_string());
            form_amount.set("".to_string());
            form_description.set("".to_string());
        })
    };

    html! {
        { page_shell(
            "Income Tracker",
            html! {},
            html! {
                <>
                    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-stretch">
                <div class="lg:col-span-4 bg-white p-5 rounded-[10px] shadow-sm border border-white/50 flex flex-col justify-center">
                    <div class="flex items-center gap-2 mb-1">
                        <div class="p-1.5 bg-[#f1f5f9] rounded-lg">{ icon_wallet() }</div>
                        <span class="text-muted-foreground text-[10px] font-bold mb-1 tracking-widest">{"Total Available Balance"}</span>
                    </div>
                    <h3 class="text-2xl font-bold text-[#1D617A] tracking-tight">{ format_currency(total_balance, &currency_symbol) }</h3>
                </div>

                <div class="lg:col-span-8 bg-white p-5 rounded-[10px] shadow-sm border border-white/50">
                    <h4 class="text-[#1D617A] font-bold text-[15px] mb-3 tracking-wider">{"Add New Income"}</h4>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
                        <div class="space-y-1">
                            <label class="text-[12px] font-bold text-muted-foreground">{"Date"}</label>
                            <input type="date" value={(*form_date).clone()} oninput={{
                                let form_date = form_date.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    form_date.set(input.value());
                                })
                            }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                        </div>
                        <div class="space-y-1">
                            <label class="text-[12px] font-bold text-muted-foreground">{ format!("Amount ({})", currency_symbol) }</label>
                            <input type="number" placeholder={format!("{} 0.00", currency_symbol)} value={(*form_amount).clone()} oninput={{
                                let form_amount = form_amount.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    form_amount.set(input.value());
                                })
                            }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                        </div>
                        <div class="space-y-1">
                            <label class="text-[12px] font-bold text-muted-foreground">{"Description"}</label>
                            <input type="text" placeholder="Income source" value={(*form_description).clone()} oninput={{
                                let form_description = form_description.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    form_description.set(input.value());
                                })
                            }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                        </div>
                        <div class="space-y-1">
                            <label class="text-[12px] font-bold text-muted-foreground">{"Category"}</label>
                            <select value={(*form_category).clone()} onchange={{
                                let form_category = form_category.clone();
                                Callback::from(move |e: Event| {
                                    let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                    form_category.set(input.value());
                                })
                            }} class="w-full bg-[#f1f4f9] border-2 border-transparent rounded-[10px] px-3 py-2 text-[11px] focus:ring-2 focus:ring-[#1D617A] outline-none">
                                <option>{"Salary"}</option>
                                <option>{"Freelance"}</option>
                                <option>{"Investment"}</option>
                            </select>
                        </div>
                    </div>
                    <div class="flex gap-3">
                        <button onclick={on_add} class="flex-1 bg-[#173E63] text-white py-2 rounded-[10px] text-[10px] font-bold flex items-center justify-center gap-2" disabled={*saving}>{ if *saving { "Saving..." } else { "Add Income" } }</button>
                        <button onclick={on_clear} class="flex-1 bg-[#B2CBDE] text-[#173E63] py-2 rounded-[10px] text-[10px] font-bold flex items-center justify-center gap-2">{"Clear"}</button>
                    </div>
                    {
                        if let Some(msg) = &*form_error {
                            html! { <p class="text-sm text-red-500 mt-3">{ msg.clone() }</p> }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </div>
                    <div class="bg-white rounded-[10px] shadow-sm border border-white/50 overflow-hidden">
                        <div class="p-5 border-b border-border">
                            <h3 class="font-bold text-lg text-foreground">{"Income History"}</h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-muted text-muted-foreground text-[10px] uppercase tracking-widest">
                                        <th class="px-8 py-4 font-bold">{"Date"}</th>
                                        <th class="px-8 py-4 font-bold">{"Description"}</th>
                                        <th class="px-8 py-4 font-bold">{"Category"}</th>
                                        <th class="px-8 py-4 font-bold text-right">{"Amount"}</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-border">
                                    { if *loading {
                                        html! { <tr><td colspan="4" class="px-8 py-6 text-center text-muted-foreground">{"Loading..."}</td></tr> }
                                    } else if incomes.is_empty() {
                                        html! { <tr><td colspan="4" class="px-8 py-6 text-center text-muted-foreground">{"No income transactions yet."}</td></tr> }
                                    } else {
                                        html! {
                                            <>
                                                { for incomes.iter().enumerate().map(|(idx, item)| html! {
                                                    <tr key={idx} class="text-sm hover:bg-muted/40 transition-colors group">
                                                        <td class="px-8 py-4 text-muted-foreground">{ item.date.clone() }</td>
                                                        <td class="px-8 py-4 text-foreground">{ item.description.clone() }</td>
                                                        <td class="px-6 py-4">
                                                            <span class="bg-secondary text-secondary-foreground px-2.5 py-1 rounded-md text-[9px] font-bold">{ item.category.clone() }</span>
                                                        </td>
                                                        <td class="px-6 py-4 text-right font-semibold text-foreground">{ format!("+ {}", format_currency(item.amount, &currency_symbol)) }</td>
                                                    </tr>
                                                }) }
                                            </>
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Contribution {
    date: String,
    description: String,
    amount: i64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct SavingGoalState {
    title: String,
    target_amount: i64,
    target_date: String,
    contributions: Vec<Contribution>,
}

fn load_saving_goal() -> SavingGoalState {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(raw)) = storage.get_item("saving_goal") {
                if let Ok(goal) = serde_json::from_str::<SavingGoalState>(&raw) {
                    return goal;
                }
            }
        }
    }

    SavingGoalState {
        title: "New Goal".to_string(),
        target_amount: 0,
        target_date: "".to_string(),
        contributions: vec![],
    }
}

fn save_saving_goal(goal: &SavingGoalState) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(raw) = serde_json::to_string(goal) {
                let _ = storage.set_item("saving_goal", &raw);
            }
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct BudgetItem {
    category: String,
    limit: i64,
}

fn load_budgets() -> Vec<BudgetItem> {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(raw)) = storage.get_item("budgets") {
                if let Ok(items) = serde_json::from_str::<Vec<BudgetItem>>(&raw) {
                    return items;
                }
            }
        }
    }

    vec![]
}

fn save_budgets(items: &Vec<BudgetItem>) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(raw) = serde_json::to_string(items) {
                let _ = storage.set_item("budgets", &raw);
            }
        }
    }
}

#[function_component(ExpensePage)]
fn expense_page() -> Html {
    let settings = use_context::<UseStateHandle<AppSettings>>();
    let currency_symbol = settings
        .as_ref()
        .map(|s| s.currency_symbol.clone())
        .unwrap_or_else(|| "₱".to_string());
    let expenses = use_state(|| Vec::<Transaction>::new());
    let loading = use_state(|| true);

    let form_date = use_state(|| "".to_string());
    let form_amount = use_state(|| "".to_string());
    let form_category = use_state(|| "Transportation".to_string());
    let form_description = use_state(|| "".to_string());
    let form_error = use_state(|| None::<String>);
    let saving = use_state(|| false);

    {
        let expenses = expenses.clone();
        let loading = loading.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let url = format!("{}/api/transactions", API_BASE_URL);
                    let mut req = Request::get(&url).credentials(RequestCredentials::Include);
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            if let Ok(token_opt) = storage.get_item("access_token") {
                                if let Some(token) = token_opt {
                                    req = req.header("Authorization", &format!("Bearer {}", token));
                                }
                            }
                        }
                    }

                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(list) = resp.json::<Vec<Transaction>>().await {
                                let filtered = list
                                    .into_iter()
                                    .filter(|t| t.amount < 0)
                                    .collect::<Vec<_>>();
                                expenses.set(filtered);
                            }
                        }
                    }
                    loading.set(false);
                });
                || ()
            },
            (),
        );
    }

    let total_expense: i64 = expenses.iter().map(|item| item.amount.abs()).sum();

    let on_add = {
        let expenses = expenses.clone();
        let form_date = form_date.clone();
        let form_amount = form_amount.clone();
        let form_category = form_category.clone();
        let form_description = form_description.clone();
        let form_error = form_error.clone();
        let saving = saving.clone();
        Callback::from(move |_| {
            let date_val = form_date.trim().to_string();
            let desc_val = form_description.trim().to_string();
            let cat_val = form_category.trim().to_string();
            let amt_val = form_amount.trim().to_string();

            if date_val.is_empty()
                || desc_val.is_empty()
                || cat_val.is_empty()
                || amt_val.is_empty()
            {
                form_error.set(Some("Please complete all fields.".to_string()));
                return;
            }

            let parsed = amt_val.parse::<i64>().unwrap_or(0);
            if parsed <= 0 {
                form_error.set(Some("Amount must be a positive number.".to_string()));
                return;
            }

            form_error.set(None);
            saving.set(true);

            let expenses = expenses.clone();
            let form_date = form_date.clone();
            let form_amount = form_amount.clone();
            let form_category = form_category.clone();
            let form_description = form_description.clone();
            let saving = saving.clone();
            spawn_local(async move {
                let url = format!("{}/api/transactions", API_BASE_URL);
                let payload = serde_json::json!({
                    "date": date_val.as_str(),
                    "description": desc_val.as_str(),
                    "category": cat_val.as_str(),
                    "amount": -parsed
                });

                let mut builder = Request::post(&url).credentials(RequestCredentials::Include);
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(token_opt) = storage.get_item("access_token") {
                            if let Some(token) = token_opt {
                                builder =
                                    builder.header("Authorization", &format!("Bearer {}", token));
                            }
                        }
                    }
                }

                let builder = match builder.json(&payload) {
                    Ok(b) => b,
                    Err(_) => return,
                };

                if let Ok(resp) = builder.send().await {
                    if resp.ok() {
                        if let Ok(created) = resp.json::<Transaction>().await {
                            let mut next = (*expenses).clone();
                            next.insert(0, created);
                            expenses.set(next);
                            form_date.set("".to_string());
                            form_amount.set("".to_string());
                            form_category.set("Transportation".to_string());
                            form_description.set("".to_string());
                        }
                    }
                }
                saving.set(false);
            });
        })
    };

    let on_clear = {
        let form_amount = form_amount.clone();
        let form_description = form_description.clone();
        let form_date = form_date.clone();
        Callback::from(move |_| {
            form_date.set("".to_string());
            form_amount.set("".to_string());
            form_description.set("".to_string());
        })
    };

    html! {
        { page_shell(
            "Expense Tracker",
            html! {},
            html! {
                <>
                    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
                        <div class="lg:col-span-4 bg-white p-5 rounded-[10px] shadow-sm border border-white/50 flex flex-col justify-center">
                            <div class="flex items-center gap-2 mb-1">
                                <div class="p-1.5 bg-[#f1f5f9] rounded-lg">{ icon_credit_card() }</div>
                                <span class="text-muted-foreground text-[10px] font-bold mb-1 tracking-widest">{"Total Expenses"}</span>
                            </div>
                            <h3 class="text-2xl font-bold text-[#1D617A] tracking-tight">{ format_currency(total_expense, &currency_symbol) }</h3>
                        </div>

                        <div class="lg:col-span-8 bg-white p-5 rounded-[10px] shadow-sm border border-white/50">
                            <h4 class="text-[#1D617A] font-bold text-[15px] mb-3 tracking-wider">{"Add New Expense"}</h4>
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
                                <div class="space-y-1">
                                    <label class="text-[12px] font-bold text-muted-foreground">{"Date"}</label>
                                    <input type="date" value={(*form_date).clone()} oninput={{
                                        let form_date = form_date.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            form_date.set(input.value());
                                        })
                                    }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-[12px] font-bold text-muted-foreground">{ format!("Amount ({})", currency_symbol) }</label>
                                    <input type="number" placeholder={format!("{} 0.00", currency_symbol)} value={(*form_amount).clone()} oninput={{
                                        let form_amount = form_amount.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            form_amount.set(input.value());
                                        })
                                    }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-[12px] font-bold text-muted-foreground">{"Description"}</label>
                                    <input type="text" placeholder="Expense description" value={(*form_description).clone()} oninput={{
                                        let form_description = form_description.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            form_description.set(input.value());
                                        })
                                    }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-[12px] font-bold text-muted-foreground">{"Category"}</label>
                                    <input type="text" placeholder="Category" value={(*form_category).clone()} oninput={{
                                        let form_category = form_category.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            form_category.set(input.value());
                                        })
                                    }} class="w-full bg-[#f1f4f9] rounded-[10px] px-3 py-2 text-[11px] text-[#173E63] border-none" />
                                </div>
                            </div>
                            <div class="flex gap-3">
                                <button onclick={on_add} class="flex-1 bg-[#173E63] text-white py-2 rounded-[10px] text-[10px] font-bold flex items-center justify-center gap-2" disabled={*saving}>{ if *saving { "Saving..." } else { "Add Expense" } }</button>
                                <button onclick={on_clear} class="flex-1 bg-[#B2CBDE] text-[#173E63] py-2 rounded-[10px] text-[10px] font-bold flex items-center justify-center gap-2">{"Clear"}</button>
                            </div>
                            {
                                if let Some(msg) = &*form_error {
                                    html! { <p class="text-sm text-red-500 mt-3">{ msg.clone() }</p> }
                                } else { html!{} }
                            }
                        </div>
                    </div>
                    <div class="bg-card rounded-2xl shadow-md border border-border overflow-hidden">
                        <div class="p-5 border-b border-border">
                            <h3 class="font-bold text-lg text-foreground">{"Expenses History"}</h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-muted text-muted-foreground text-[10px] uppercase tracking-widest">
                                        <th class="px-8 py-4 font-bold">{"Date"}</th>
                                        <th class="px-8 py-4 font-bold">{"Description"}</th>
                                        <th class="px-8 py-4 font-bold">{"Category"}</th>
                                        <th class="px-8 py-4 font-bold">{"Amount"}</th>
                                        <th class="px-8 py-4 font-bold">{"Action"}</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-border">
                                    { if *loading {
                                        html! { <tr><td colspan="5" class="px-8 py-6 text-center text-muted-foreground">{"Loading..."}</td></tr> }
                                    } else if expenses.is_empty() {
                                        html! { <tr><td colspan="5" class="px-8 py-6 text-center text-muted-foreground">{"No expense transactions yet."}</td></tr> }
                                    } else {
                                        html! {
                                            <>
                                                { for expenses.iter().enumerate().map(|(idx, item)| html! {
                                                    <tr key={idx} class="text-sm hover:bg-muted/40 transition-colors group">
                                                        <td class="px-8 py-4 text-muted-foreground">{ item.date.clone() }</td>
                                                        <td class="px-8 py-4 text-foreground">{ item.description.clone() }</td>
                                                        <td class="px-8 py-4">
                                                            <span class="bg-secondary text-secondary-foreground px-3 py-1 rounded-full text-[10px] font-bold">{ item.category.clone() }</span>
                                                        </td>
                                                        <td class="px-8 py-4 font-semibold text-foreground">{ format_currency(item.amount, &currency_symbol) }</td>
                                                        <td class="px-8 py-4"></td>
                                                    </tr>
                                                }) }
                                            </>
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

#[function_component(SavingsPage)]
fn savings_page() -> Html {
    let is_creating = use_state(|| false);
    let settings = use_context::<UseStateHandle<AppSettings>>();
    let currency_symbol = settings
        .as_ref()
        .map(|s| s.currency_symbol.clone())
        .unwrap_or_else(|| "₱".to_string());

    let goal = use_state(load_saving_goal);
    let contrib_date = use_state(|| "".to_string());
    let contrib_amount = use_state(|| "".to_string());
    let contrib_desc = use_state(|| "".to_string());
    let new_goal_title = use_state(|| "".to_string());
    let new_goal_amount = use_state(|| "".to_string());
    let new_goal_date = use_state(|| "".to_string());

    let saved_so_far: i64 = goal.contributions.iter().map(|c| c.amount).sum();
    let progress = if goal.target_amount > 0 {
        saved_so_far as f64 / goal.target_amount as f64
    } else {
        0.0
    };
    let radius = 38.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let offset = circumference - progress.min(1.0) * circumference;

    let toggle_create = {
        let is_creating = is_creating.clone();
        let goal = goal.clone();
        let new_goal_title = new_goal_title.clone();
        let new_goal_amount = new_goal_amount.clone();
        let new_goal_date = new_goal_date.clone();
        Callback::from(move |_| {
            if !*is_creating {
                new_goal_title.set(goal.title.clone());
                new_goal_amount.set(goal.target_amount.to_string());
                new_goal_date.set(goal.target_date.clone());
            }
            is_creating.set(!*is_creating)
        })
    };

    let add_contribution = {
        let goal = goal.clone();
        let contrib_date = contrib_date.clone();
        let contrib_amount = contrib_amount.clone();
        let contrib_desc = contrib_desc.clone();
        Callback::from(move |_| {
            let parsed = contrib_amount.parse::<i64>().unwrap_or(0);
            if parsed <= 0 {
                return;
            }
            let mut next_goal = (*goal).clone();
            let entry = Contribution {
                date: contrib_date.to_string(),
                description: if contrib_desc.is_empty() {
                    "Contribution".into()
                } else {
                    contrib_desc.to_string()
                },
                amount: parsed,
            };
            next_goal.contributions.insert(0, entry);
            save_saving_goal(&next_goal);
            goal.set(next_goal);
            contrib_amount.set("".into());
            contrib_desc.set("".into());

            // Also create a transaction so savings are reflected in totals
            let date_val = contrib_date.to_string();
            let desc_val = if contrib_desc.is_empty() {
                "Savings".to_string()
            } else {
                contrib_desc.to_string()
            };
            spawn_local(async move {
                let url = format!("{}/api/transactions", API_BASE_URL);
                let payload = serde_json::json!({
                    "date": date_val.as_str(),
                    "description": desc_val.as_str(),
                    "category": "Savings",
                    "amount": -parsed
                });

                let mut builder = Request::post(&url).credentials(RequestCredentials::Include);
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(token_opt) = storage.get_item("access_token") {
                            if let Some(token) = token_opt {
                                builder =
                                    builder.header("Authorization", &format!("Bearer {}", token));
                            }
                        }
                    }
                }

                if let Ok(builder) = builder.json(&payload) {
                    let _ = builder.send().await;
                }
            });
        })
    };

    let clear_contribution = {
        let contrib_amount = contrib_amount.clone();
        let contrib_desc = contrib_desc.clone();
        Callback::from(move |_| {
            contrib_amount.set("".to_string());
            contrib_desc.set("".to_string());
        })
    };

    let remove_goal = {
        let goal = goal.clone();
        let new_goal_title = new_goal_title.clone();
        let new_goal_amount = new_goal_amount.clone();
        let new_goal_date = new_goal_date.clone();
        let is_creating = is_creating.clone();
        Callback::from(move |_| {
            let cleared = SavingGoalState {
                title: "".to_string(),
                target_amount: 0,
                target_date: "".to_string(),
                contributions: vec![],
            };
            save_saving_goal(&cleared);
            goal.set(cleared);
            new_goal_title.set("".to_string());
            new_goal_amount.set("".to_string());
            new_goal_date.set("".to_string());
            is_creating.set(false);
        })
    };

    let create_goal = {
        let goal = goal.clone();
        let new_goal_title = new_goal_title.clone();
        let new_goal_amount = new_goal_amount.clone();
        let new_goal_date = new_goal_date.clone();
        let is_creating = is_creating.clone();
        Callback::from(move |_| {
            if new_goal_title.is_empty() || new_goal_amount.is_empty() {
                return;
            }
            let next_goal = SavingGoalState {
                title: new_goal_title.to_string(),
                target_amount: new_goal_amount.parse::<i64>().unwrap_or(0),
                target_date: new_goal_date.to_string(),
                contributions: vec![],
            };
            save_saving_goal(&next_goal);
            goal.set(next_goal);
            is_creating.set(false);
        })
    };

    html! {
        { page_shell(
            "Saving Goal",
            html! {
                <div class="flex items-center gap-2">
                    <button onclick={toggle_create} class="bg-primary text-primary-foreground px-4 py-2 rounded-[10px] text-xs font-bold uppercase flex items-center gap-1 shadow-md hover:opacity-90 transition-all">
                        { if *is_creating { "Cancel" } else { "New Goal" } }
                    </button>
                    <button onclick={remove_goal} class="bg-red-600 text-white px-4 py-2 rounded-[10px] text-xs font-bold uppercase shadow-md hover:opacity-90 transition-all">
                        {"Remove Goal"}
                    </button>
                </div>
            },
            html! {
                <>
                    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
                <div class="lg:col-span-5 bg-white p-6 rounded-[10px] shadow-md border border-border flex flex-col h-full">
                    { if !*is_creating {
                        html! {
                            <div class="flex flex-col h-full">
                                <div class="flex justify-between items-start mb-6">
                                    <div class="space-y-1">
                                        <div class="flex items-center gap-2 px-2 py-0.5 bg-[#dae3f0] w-fit rounded-full">
                                            <span class="text-[9px] font-black text-[#173E63] uppercase tracking-wider">{"Current Goal"}</span>
                                        </div>
                                        <h3 class="text-xl font-black text-[#173E63] tracking-tight">{ goal.title.clone() }</h3>
                                    </div>
                                    <div class="text-right">
                                        <p class="text-[9px] font-bold text-slate-400 uppercase">{"Target Date"}</p>
                                        <div class="flex items-center gap-1 text-[#173E63] font-bold text-xs">{ if goal.target_date.is_empty() { "No Date" } else { goal.target_date.as_str() } }</div>
                                    </div>
                                </div>

                                <div class="flex flex-1 items-center justify-around gap-4 bg-slate-50/50 rounded-2xl p-4 border border-slate-100/50">
                                    <div class="relative flex items-center justify-center shrink-0">
                                        <svg class="w-24 h-24 transform -rotate-90">
                                            <circle cx="48" cy="48" r={radius.to_string()} stroke="#e2e8f0" stroke-width="8" fill="transparent" />
                                            <circle cx="48" cy="48" r={radius.to_string()} stroke="#173E63" stroke-width="8" fill="transparent" stroke-dasharray={circumference.to_string()} stroke-dashoffset={offset.to_string()} stroke-linecap="round" />
                                        </svg>
                                        <div class="absolute inset-0 flex flex-col items-center justify-center">
                                            <span class="text-lg font-black text-[#173E63]">{ format!("{}%", (progress * 100.0).round() as i32) }</span>
                                            <span class="text-[7px] text-slate-400 font-bold uppercase tracking-tighter">{"Progress"}</span>
                                        </div>
                                    </div>
                                    <div class="space-y-3">
                                        <div>
                                            <p class="text-[12px] font-bold text-slate-400 mb-0.5 tracking-widest">{"Amount Saved"}</p>
                                            <p class="text-lg font-black text-[#1D617A] leading-none">{ format_currency(saved_so_far, &currency_symbol) }</p>
                                        </div>
                                        <div>
                                            <p class="text-[10px] font-bold text-slate-400 mb-0.5 tracking-widest">{"Goal Target"}</p>
                                            <p class="text-sm font-black text-[#173E63]/70 leading-none">{ format_currency(goal.target_amount, &currency_symbol) }</p>
                                        </div>
                                    </div>
                                </div>
                                {
                                    if goal.target_amount > 0 && saved_so_far >= goal.target_amount {
                                        html! {
                                            <div class="mt-4 p-3 rounded-lg bg-green-50 border border-green-200 text-green-700 text-xs font-bold">
                                                {"Goal reached! You can set a new goal or keep saving."}
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        }
                    } else {
                        html! {
                            <div class="flex flex-col h-full space-y-3">
                                <h4 class="text-[#173E63] font-black text-[13px] mb-2 uppercase tracking-wide border-b border-slate-50 pb-2 text-center">{"Setup New Goal"}</h4>
                                <div class="flex-grow space-y-3">
                                    <div class="space-y-1">
                                        <label class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{"Goal Name"}</label>
                                        <input type="text" value={(*new_goal_title).clone()} oninput={{
                                            let new_goal_title = new_goal_title.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                new_goal_title.set(input.value());
                                            })
                                        }} class="w-full bg-[#f1f4f9] border-none rounded-xl p-2.5 text-xs font-bold text-[#173E63] outline-none" placeholder="e.g. Dream Wedding" />
                                    </div>
                                    <div class="grid grid-cols-2 gap-3">
                                        <div class="space-y-1">
                                            <label class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{ format!("Amount ({})", currency_symbol) }</label>
                                            <input type="number" value={(*new_goal_amount).clone()} oninput={{
                                                let new_goal_amount = new_goal_amount.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    new_goal_amount.set(input.value());
                                                })
                                            }} class="w-full bg-[#f1f4f9] border-none rounded-xl p-2.5 text-xs font-bold text-[#173E63] outline-none" placeholder="0.00" />
                                        </div>
                                        <div class="space-y-1">
                                            <label class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{"Target Date"}</label>
                                            <input type="date" value={(*new_goal_date).clone()} oninput={{
                                                let new_goal_date = new_goal_date.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    new_goal_date.set(input.value());
                                                })
                                            }} class="w-full bg-[#f1f4f9] border-none rounded-xl p-2.5 text-xs font-bold text-[#173E63] outline-none" />
                                        </div>
                                    </div>
                                </div>
                                <button onclick={create_goal} class="w-full bg-[#1D617A] text-white py-2.5 rounded-xl text-[10px] font-black uppercase flex items-center justify-center gap-2 shadow-md">{"Start New Goal"}</button>
                            </div>
                        }
                    }}
                </div>

                <div class="lg:col-span-7 bg-white p-6 rounded-2xl shadow-md border border-border flex flex-col h-full">
                    <h4 class="text-[#1D617A] font-bold text-[13px] mb-6 uppercase tracking-widest border-b border-slate-50 pb-2">{"Add Contribution"}</h4>
                    <div class="flex-grow space-y-5">
                        <div class="grid grid-cols-2 gap-4">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{"Date"}</label>
                                <input type="date" value={(*contrib_date).clone()} oninput={{
                                    let contrib_date = contrib_date.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        contrib_date.set(input.value());
                                    })
                                }} class="w-full bg-[#f1f4f9] border-none rounded-xl p-3 text-xs font-bold text-[#173E63] transition-all" />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{ format!("Amount ({})", currency_symbol) }</label>
                                <input type="number" placeholder={format!("{} 0.00", currency_symbol)} value={(*contrib_amount).clone()} oninput={{
                                    let contrib_amount = contrib_amount.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        contrib_amount.set(input.value());
                                    })
                                }} class="w-full bg-[#f1f4f9] border-none rounded-xl p-3 text-xs font-bold text-[#173E63] transition-all" />
                            </div>
                        </div>
                        <div class="space-y-1.5">
                            <label class="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{"Description"}</label>
                            <input type="text" placeholder="e.g. Monthly Savings" value={(*contrib_desc).clone()} oninput={{
                                let contrib_desc = contrib_desc.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    contrib_desc.set(input.value());
                                })
                            }} class="w-full bg-[#f1f4f9] border-none rounded-xl p-3 text-xs font-bold text-[#173E63] transition-all" />
                        </div>
                    </div>
                    <div class="flex gap-3 mt-8">
                        <button onclick={add_contribution} class="flex-2 grow-[2] bg-[#173E63] text-white py-3 rounded-[10px] text-[10px] font-bold flex items-center justify-center gap-2 shadow-md hover:translate-y-[-1px] transition-all">{"Add Contribution"}</button>
                        <button onclick={clear_contribution} class="flex-1 bg-[#D8E1E8] text-[#173E63] py-3 rounded-[10px] text-[10px] font-bold flex items-center justify-center gap-2">{"Clear"}</button>
                    </div>
                </div>
            </div>
                    <div class="bg-white rounded-2xl shadow-md border border-border overflow-hidden">
                        <div class="p-5 border-b border-border">
                            <h3 class="font-bold text-foreground text-lg">{"Contribution History"}</h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-muted text-muted-foreground text-[10px] uppercase tracking-widest">
                                        <th class="px-8 py-4 font-bold">{"Date"}</th>
                                        <th class="px-8 py-4 font-bold">{"Description"}</th>
                                        <th class="px-8 py-4 font-bold text-right">{"Amount"}</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-border">
                                    { for goal.contributions.iter().enumerate().map(|(idx, item)| html! {
                                        <tr key={idx} class="text-sm hover:bg-muted/40 transition-colors">
                                            <td class="px-8 py-4 text-muted-foreground">{ item.date.clone() }</td>
                                            <td class="px-8 py-4 text-foreground">{ item.description.clone() }</td>
                                            <td class="px-8 py-4 text-right font-semibold text-foreground">{ format_currency(item.amount, &currency_symbol) }</td>
                                        </tr>
                                    }) }
                                </tbody>
                            </table>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

#[function_component(SummaryPage)]
fn summary_page() -> Html {
    let settings = use_context::<UseStateHandle<AppSettings>>();
    let currency_symbol = settings
        .as_ref()
        .map(|s| s.currency_symbol.clone())
        .unwrap_or_else(|| "₱".to_string());

    let total_income = use_state(|| 0i64);
    let total_expenses = use_state(|| 0i64);
    let balance = use_state(|| 0i64);
    let recent = use_state(|| Vec::<Transaction>::new());
    let loading = use_state(|| true);

    {
        let total_income = total_income.clone();
        let total_expenses = total_expenses.clone();
        let balance = balance.clone();
        let recent = recent.clone();
        let loading = loading.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let summary_url = format!("{}/api/dashboard/summary", API_BASE_URL);
                    let mut req =
                        Request::get(&summary_url).credentials(RequestCredentials::Include);
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            if let Ok(token_opt) = storage.get_item("access_token") {
                                if let Some(token) = token_opt {
                                    req = req.header("Authorization", &format!("Bearer {}", token));
                                }
                            }
                        }
                    }

                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(v) = json.get("total_income").and_then(|x| x.as_i64()) {
                                    total_income.set(v);
                                }
                                if let Some(v) = json.get("total_expenses").and_then(|x| x.as_i64())
                                {
                                    total_expenses.set(v);
                                }
                                if let Some(v) = json.get("balance").and_then(|x| x.as_i64()) {
                                    balance.set(v);
                                }
                            }
                        }
                    }

                    let tx_url = format!("{}/api/transactions", API_BASE_URL);
                    let mut req2 = Request::get(&tx_url).credentials(RequestCredentials::Include);
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            if let Ok(token_opt) = storage.get_item("access_token") {
                                if let Some(token) = token_opt {
                                    req2 =
                                        req2.header("Authorization", &format!("Bearer {}", token));
                                }
                            }
                        }
                    }
                    if let Ok(resp2) = req2.send().await {
                        if resp2.ok() {
                            if let Ok(list) = resp2.json::<Vec<Transaction>>().await {
                                recent.set(list.into_iter().take(10).collect());
                            }
                        }
                    }

                    loading.set(false);
                });
                || ()
            },
            (),
        );
    }

    html! {
        { page_shell(
            "Summary Report",
            html! {},
            html! {
                <>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div class="bg-card rounded-lg p-6 border border-border">
                            <p class="text-sm text-muted-foreground mb-2">{"Total Income"}</p>
                            <h3 class="text-3xl font-bold text-foreground">{ format_currency(*total_income, &currency_symbol) }</h3>
                        </div>
                        <div class="bg-card rounded-lg p-6 border border-border">
                            <p class="text-sm text-muted-foreground mb-2">{"Total Expenses"}</p>
                            <h3 class="text-3xl font-bold text-foreground">{ format_currency(*total_expenses, &currency_symbol) }</h3>
                        </div>
                        <div class="bg-card rounded-lg p-6 border border-border">
                            <p class="text-sm text-muted-foreground mb-2">{"Net Balance"}</p>
                            <h3 class="text-3xl font-bold text-foreground">{ format_currency(*balance, &currency_symbol) }</h3>
                        </div>
                    </div>

                    <div class="bg-card rounded-lg border border-border overflow-hidden">
                        <div class="px-6 py-4 border-b border-border">
                            <h3 class="text-lg font-bold text-foreground">{"Recent Transactions"}</h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead class="bg-secondary border-b border-border">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-sm font-semibold text-secondary-foreground">{"Date"}</th>
                                        <th class="px-6 py-3 text-left text-sm font-semibold text-secondary-foreground">{"Description"}</th>
                                        <th class="px-6 py-3 text-left text-sm font-semibold text-secondary-foreground">{"Category"}</th>
                                        <th class="px-6 py-3 text-right text-sm font-semibold text-secondary-foreground">{"Amount"}</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-border">
                                    { if *loading {
                                        html! { <tr><td colspan="4" class="px-6 py-6 text-center text-muted-foreground">{"Loading..."}</td></tr> }
                                    } else if recent.is_empty() {
                                        html! { <tr><td colspan="4" class="px-6 py-6 text-center text-muted-foreground">{"No transactions yet."}</td></tr> }
                                    } else {
                                        html! {
                                            <>
                                                { for recent.iter().map(|tx| html! {
                                                    <tr class="text-sm hover:bg-muted/30 transition-colors">
                                                        <td class="px-6 py-3 text-muted-foreground">{ tx.date.clone() }</td>
                                                        <td class="px-6 py-3 text-foreground">{ tx.description.clone() }</td>
                                                        <td class="px-6 py-3 text-foreground">{ tx.category.clone() }</td>
                                                        <td class="px-6 py-3 text-right font-semibold text-foreground">{ format_currency(tx.amount, &currency_symbol) }</td>
                                                    </tr>
                                                }) }
                                            </>
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

#[function_component(SettingsPage)]
fn settings_page() -> Html {
    let settings = use_context::<UseStateHandle<AppSettings>>();
    let budget_alerts = use_state(|| true);
    let monthly_report = use_state(|| true);
    let saving_alert = use_state(|| true);

    let current_currency = settings
        .as_ref()
        .map(|s| s.currency_code.clone())
        .unwrap_or_else(|| "PHP".to_string());

    let on_currency_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            if let Some(settings) = settings.as_ref() {
                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                let code = input.value();
                let symbol = currency_symbol_for(&code).to_string();
                let next = AppSettings {
                    currency_code: code,
                    currency_symbol: symbol,
                };
                save_settings(&next);
                settings.set(next);
            }
        })
    };

    html! {
        { page_shell(
            "Settings",
            html! {},
            html! {
                <>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div class="bg-card rounded-lg p-6 border border-border">
                            <h2 class="text-xl font-bold text-foreground mb-6">{"Preferences"}</h2>
                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-foreground mb-2">{"Currency"}</label>
                                    <select value={current_currency} onchange={on_currency_change} class="w-full px-4 py-2 bg-input border border-input rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-primary">
                                        <option value="PHP">{"PHP (₱)"}</option>
                                        <option value="USD">{"USD ($)"}</option>
                                        <option value="EUR">{"EUR (€)"}</option>
                                        <option value="GBP">{"GBP (£)"}</option>
                                        <option value="JPY">{"JPY (¥)"}</option>
                                    </select>
                                    <p class="text-xs text-muted-foreground mt-2">{"Currency updates are applied across the dashboard and reports."}</p>
                                </div>
                            </div>
                        </div>

                        <div class="bg-card rounded-lg p-6 border border-border">
                            <h2 class="text-xl font-bold text-foreground mb-6">{"Notifications"}</h2>
                            <div class="space-y-4">
                                <div class="flex items-start gap-3 pb-4 border-b border-border">
                                    <div class="flex-1 pt-1">
                                        <p class="font-medium text-foreground">{"Budget Alerts"}</p>
                                        <p class="text-sm text-muted-foreground">{"Get notified when expenses exceed your plan"}</p>
                                    </div>
                                    <input type="checkbox" checked={*budget_alerts} onclick={{
                                        let budget_alerts = budget_alerts.clone();
                                        Callback::from(move |_| budget_alerts.set(!*budget_alerts))
                                    }} />
                                </div>
                                <div class="flex items-start gap-3 pb-4 border-b border-border">
                                    <div class="flex-1 pt-1">
                                        <p class="font-medium text-foreground">{"Monthly Report"}</p>
                                        <p class="text-sm text-muted-foreground">{"Receive a summary of your spending monthly"}</p>
                                    </div>
                                    <input type="checkbox" checked={*monthly_report} onclick={{
                                        let monthly_report = monthly_report.clone();
                                        Callback::from(move |_| monthly_report.set(!*monthly_report))
                                    }} />
                                </div>
                                <div class="flex items-start gap-3">
                                    <div class="flex-1 pt-1">
                                        <p class="font-medium text-foreground">{"Saving Alert"}</p>
                                        <p class="text-sm text-muted-foreground">{"Get notified when you reach a saving milestone"}</p>
                                    </div>
                                    <input type="checkbox" checked={*saving_alert} onclick={{
                                        let saving_alert = saving_alert.clone();
                                        Callback::from(move |_| saving_alert.set(!*saving_alert))
                                    }} />
                                </div>
                            </div>
                        </div>
                    </div>
                </>
            }
        ) }
    }
}

#[derive(Properties, PartialEq)]
struct StatCardProps {
    title: &'static str,
    amount: i64,
    icon: StatIcon,
    currency_symbol: String,
}

#[function_component(StatCard)]
fn stat_card(props: &StatCardProps) -> Html {
    html! {
        <div class="bg-card p-6 rounded-[10px] shadow-sm border border-border flex justify-between items-start">
            <div>
                <p class="text-muted-foreground text-[10px] font-bold mb-1 tracking-widest">{ props.title }</p>
                <h3 class="text-2xl font-bold text-[#1D617A] tracking-tight">{ format_currency(props.amount, &props.currency_symbol) }</h3>
            </div>
            <div class="p-3 bg-[#eef4f9] rounded-[10px]">
                {
                    match props.icon {
                        StatIcon::UpRight => icon_arrow_up_right(),
                        StatIcon::CreditCard => icon_credit_card(),
                        StatIcon::Wallet => icon_wallet(),
                    }
                }
            </div>
        </div>
    }
}

fn format_with_commas(value: i64) -> String {
    let is_negative = value < 0;
    let s = value.abs().to_string().chars().rev().collect::<Vec<char>>();
    let mut out = Vec::new();
    for (i, ch) in s.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(*ch);
    }
    let formatted: String = out.into_iter().rev().collect();
    if is_negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

fn format_currency(amount: i64, symbol: &str) -> String {
    let sign = if amount < 0 { "-" } else { "" };
    format!("{}{} {}.00", sign, symbol, format_with_commas(amount.abs()))
}

#[function_component(App)]
fn app() -> Html {
    let active_page = use_state(|| Page::Dashboard);
    let auth_status = use_state(|| AuthStatus::Checking);
    let settings = use_state(load_settings);
    let on_select = {
        let active_page = active_page.clone();
        Callback::from(move |page: Page| active_page.set(page))
    };

    {
        let auth_status = auth_status.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let url = format!("{}/api/auth/refresh", API_BASE_URL);
                    let response = Request::post(&url)
                        .credentials(RequestCredentials::Include)
                        .send()
                        .await;

                    match response {
                        Ok(resp) if resp.ok() => {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(token) =
                                    json.get("access_token").and_then(|v| v.as_str())
                                {
                                    if let Some(window) = web_sys::window() {
                                        if let Ok(Some(storage)) = window.local_storage() {
                                            let _ = storage.set_item("access_token", token);
                                        }
                                    }
                                }
                            }
                            auth_status.set(AuthStatus::Authenticated);
                        }
                        _ => {
                            // Fallback to existing access token (keeps user logged in on refresh)
                            let mut has_token = false;
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    if let Ok(token_opt) = storage.get_item("access_token") {
                                        if let Some(token) = token_opt {
                                            if !token.is_empty() {
                                                has_token = true;
                                            }
                                        }
                                    }
                                }
                            }

                            if has_token {
                                auth_status.set(AuthStatus::Authenticated);
                            } else {
                                auth_status.set(AuthStatus::Unauthenticated);
                            }
                        }
                    }
                });
                || ()
            },
            (),
        );
    }

    let content = match *active_page {
        Page::Dashboard => html! { <DashboardPage /> },
        Page::Budget => html! { <BudgetPage /> },
        Page::Income => html! { <IncomePage /> },
        Page::Expense => html! { <ExpensePage /> },
        Page::Savings => html! { <SavingsPage /> },
        Page::Summary => html! { <SummaryPage /> },
        Page::Settings => html! { <SettingsPage /> },
    };

    if *auth_status == AuthStatus::Checking {
        return html! {
            <div class="min-h-screen flex items-center justify-center bg-background text-muted-foreground">
                {"Checking session..."}
            </div>
        };
    }

    if *auth_status == AuthStatus::Unauthenticated {
        return html! { <AuthScreen on_authenticated={Callback::from(move |_| auth_status.set(AuthStatus::Authenticated))} /> };
    }

    html! {
        <ContextProvider<UseStateHandle<AppSettings>> context={settings}>
            <Layout active_page={*active_page} on_select={on_select}>
                { content }
            </Layout>
        </ContextProvider<UseStateHandle<AppSettings>>>
    }
}

#[derive(Properties, PartialEq)]
struct AuthScreenProps {
    on_authenticated: Callback<()>,
}

#[function_component(AuthScreen)]
fn auth_screen(props: &AuthScreenProps) -> Html {
    let is_login = use_state(|| true);
    let email = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let confirm_password = use_state(|| "".to_string());
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let on_submit = {
        let is_login = is_login.clone();
        let email = email.clone();
        let password = password.clone();
        let error = error.clone();
        let loading = loading.clone();
        let on_authenticated = props.on_authenticated.clone();
        let confirm_password = confirm_password.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let email_val = (*email).clone();
            let password_val = (*password).clone();
            let confirm_val = (*confirm_password).clone();
            let on_authenticated = on_authenticated.clone();

            if email_val.is_empty() || password_val.is_empty() {
                error.set(Some("Email and password are required".to_string()));
                return;
            }

            if password_val.len() < 8 {
                error.set(Some("Password must be at least 8 characters".to_string()));
                return;
            }

            if !*is_login && password_val != confirm_val {
                error.set(Some("Passwords do not match".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);

            let endpoint = if *is_login {
                "/api/auth/login"
            } else {
                "/api/auth/register"
            };
            let url = format!("{}{}", API_BASE_URL, endpoint);
            let error_async = error.clone();
            let loading_async = loading.clone();
            spawn_local(async move {
                let body = serde_json::json!({
                    "email": email_val,
                    "password": password_val,
                    "confirmPassword": confirm_val,
                });

                let response = Request::post(&url)
                    .header("Content-Type", "application/json")
                    .credentials(RequestCredentials::Include)
                    .body(serde_json::to_string(&body).unwrap())
                    .unwrap()
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.ok() => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(token) = json.get("access_token").and_then(|v| v.as_str()) {
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.set_item("access_token", token);
                                    }
                                }
                            }
                        }
                        on_authenticated.emit(());
                    }
                    Ok(resp) => {
                        let msg = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "Login failed".to_string());
                        error_async.set(Some(msg));
                    }
                    Err(_) => {
                        error_async.set(Some("Network error".to_string()));
                    }
                }
                loading_async.set(false);
            });
        })
    };

    let toggle_mode = {
        let is_login = is_login.clone();
        Callback::from(move |_| is_login.set(!*is_login))
    };

    html! {
        <div class="min-h-screen flex items-center justify-center bg-background">
            <div class="w-full max-w-md bg-card border border-border rounded-2xl shadow-lg p-8">
                <div class="text-center mb-6">
                    <h1 class="text-2xl font-bold text-foreground">{ if *is_login { "Welcome back" } else { "Create account" } }</h1>
                    <p class="text-sm text-muted-foreground mt-2">
                        { if *is_login { "Sign in to continue." } else { "Start managing your finances." } }
                    </p>
                </div>

                <form class="space-y-4" onsubmit={on_submit}>
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-foreground">{"Email"}</label>
                        <input
                            type="email"
                            class="w-full px-4 py-2 bg-input border border-input rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                            value={(*email).clone()}
                            oninput={{
                                let email = email.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    email.set(input.value());
                                })
                            }}
                        />
                    </div>
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-foreground">{"Password"}</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 bg-input border border-input rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                            value={(*password).clone()}
                            oninput={{
                                let password = password.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    password.set(input.value());
                                })
                            }}
                        />
                    </div>

                    if !*is_login {
                        <div class="space-y-1">
                            <label class="text-sm font-medium text-foreground">{"Confirm Password"}</label>
                            <input
                                type="password"
                                class="w-full px-4 py-2 bg-input border border-input rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                                value={(*confirm_password).clone()}
                                oninput={{
                                    let confirm_password = confirm_password.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        confirm_password.set(input.value());
                                    })
                                }}
                            />
                        </div>
                    }

                    if let Some(msg) = &*error {
                        <div class="text-sm text-red-500">{ msg.clone() }</div>
                    }

                    <button
                        type="submit"
                        class="w-full bg-primary text-primary-foreground py-2 rounded-lg font-semibold hover:opacity-90 transition-opacity"
                        disabled={*loading}
                    >
                        { if *loading { "Please wait..." } else if *is_login { "Login" } else { "Sign up" } }
                    </button>
                </form>

                <div class="mt-6 text-center text-sm text-muted-foreground">
                    { if *is_login { "No account?" } else { "Already have an account?" } }
                    <button class="ml-2 text-primary font-semibold" onclick={toggle_mode}>
                        { if *is_login { "Sign up" } else { "Login" } }
                    </button>
                </div>
            </div>
        </div>
    }
}

fn icon_base(path: &'static str) -> Html {
    html! {
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-foreground">
            <path d={path}></path>
        </svg>
    }
}

fn icon_bell() -> Html {
    icon_base("M18 8a6 6 0 10-12 0c0 7-3 7-3 7h18s-3 0-3-7")
}
fn icon_moon() -> Html {
    icon_base("M21 12.79A9 9 0 1111.21 3a7 7 0 109.79 9.79z")
}
fn icon_chevron_down() -> Html {
    icon_base("M6 9l6 6 6-6")
}
fn icon_layout_grid() -> Html {
    icon_base("M3 3h8v8H3zM13 3h8v8h-8zM3 13h8v8H3zM13 13h8v8h-8z")
}
fn icon_wallet() -> Html {
    icon_base("M3 7h18v10H3zM16 7V5H5v2")
}
fn icon_trending_up() -> Html {
    icon_base("M3 17l6-6 4 4 7-7")
}
fn icon_credit_card() -> Html {
    icon_base("M3 7h18v10H3zM3 11h18")
}
fn icon_target() -> Html {
    icon_base("M12 12m-9 0a9 9 0 1018 0 9 9 0 10-18 0")
}
fn icon_bar_chart() -> Html {
    icon_base("M4 20V10M10 20V4M16 20v-6M22 20H2")
}
fn icon_settings() -> Html {
    icon_base("M12 1v3M12 20v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M1 12h3M20 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1")
}
fn icon_log_out() -> Html {
    icon_base("M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9")
}
fn icon_plus() -> Html {
    icon_base("M12 5v14M5 12h14")
}
fn icon_arrow_up_right() -> Html {
    icon_base("M7 17L17 7M7 7h10v10")
}

fn main() {
    yew::Renderer::<App>::new().render();
}
