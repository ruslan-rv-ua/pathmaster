# PathMaster v0.1.0 — source PRD (input, verbatim)

> This is the **input** to the wayfinder effort, captured verbatim as the user wrote it (Ukrainian).
> It is not the deliverable. The deliverable is `spec.md` — the locked English specification produced
> by the ticket **Locked v0.1.0 spec** once every decision on the map is settled.
> Where this document and a resolved ticket disagree, the resolved ticket wins.

---

## 1. Опис проєкту

**PathMaster** — портативний Windows-застосунок для перегляду, редагування та діагностики змінної середовища `PATH`. Розповсюджується як єдиний `.exe` файл без інсталятора. Усі дані зберігаються поряд з виконуваним файлом у підпапці `data/`. Не залишає жодних слідів у реєстрі Windows, AppData або інших системних розташуваннях.

**Стек:** rust + https://github.com/AllenDang/wxdragon

**Перша публічна версія MVP:** `v0.1.0`

---

## 2. Цілі

### Основна мета
Замінити незручний штатний редактор PATH Windows повнофункціональним портативним інструментом, який підтримує автодіагностику, бекапи та відповідає стандартам доступності NVDA.

### Вторинні цілі
- Забезпечити безпечне редагування PATH з автоматичними бекапами перед кожною зміною.
- Надати однакову якість використання для зрячих і незрячих користувачів.
- Підтримати i18n (English / Ukrainian) без перезапуску. У майбутньому інші локалізації.

---

## 3. Цільова аудиторія

| Сегмент | Потреба |
|---|---|
| Розробники та DevOps | Безліч PATH-записів від IDE, SDK, CLI-утиліт; потрібна швидка діагностика дублікатів і некоректних шляхів |
| Користувачі з порушеннями зору | Повна навігація клавіатурою, підтримка NVDA screen reader, оголошення статусів записів |
| Звичайні користувачі Windows | Проста та зрозуміла альтернатива системному діалогу редагування змінних середовища |

---

## 4. Припущення

- Цільова ОС: Windows 10 21H2+ та Windows 11; 32-bit не підтримується.
- Редагування System PATH потребує прав адміністратора; UAC elevation ініціюється з програми.
- Користувач має стандартні права для читання та запису User PATH.
- NVDA — єдиний screen reader у scope; інші (JAWS, Narrator) не тестуються цілеспрямовано, але не мають навмисно блокуватися.
- Файл `data/settings.json` відсутній при першому запуску — програма застосовує дефолтні значення і створює файл автоматично.

---

## 5. Користувацькі сторії

### US-view-path
**Priority:** 🔴 must
**Description:** Як користувач, я хочу бачити всі записи User PATH і System PATH у вигляді списку з індексами, шляхами та статусами, щоб швидко оцінити стан змінної середовища.
**Acceptance:**
- Інтерфейс містить 3 вкладки: "User PATH", "System PATH", "Backups".
- Кожен рядок містить: порядковий індекс, повний шлях, статус (OK / Warning / Error).
- Список відображається протягом 2 секунд після запуску на SSD.

### US-diagnose
**Priority:** 🔴 must
**Description:** Як користувач, я хочу, щоб застосунок автоматично знаходив проблеми в PATH (дублікати, неіснуючі шляхи тощо), щоб я міг їх усунути без ручної перевірки.
**Acceptance:**
- Діагностика запускається автоматично при завантаженні та після кожної зміни.
- Виявляються всі 6 типів проблем: дублікати, неіснуючі шляхи, перевищення 32767 символів, підозрілі схожі шляхи (typos), відносні шляхи, порожні записи.
- Статус кожного запису відображається текстом і/або іконкою — колір не є єдиним індикатором.

### US-edit
**Priority:** 🔴 must
**Description:** Як користувач, я хочу додавати, редагувати, видаляти та змінювати порядок записів PATH, щоб керувати змінною.
**Acceptance:**
- Редагування відкривається по F2 або подвійному кліку на запис.
- Доступні дії: Додати, Видалити, Move Up, Move Down, Drag & Drop.
- Ctrl+Z скасовує останню дію; Ctrl+Y повертає.
- Apply зберігає зміни в реєстр і надсилає `WM_SETTINGCHANGE`; Cancel скасовує незбережені зміни.
- Move Up / Move Down доступні з клавіатури (🔴 must); Drag & Drop (🟡 should).

### US-admin-elevation
**Priority:** 🔴 must
**Description:** Як звичайний користувач без прав адміністратора, я хочу бачити чітке повідомлення і отримати пропозицію підвищити права через UAC, коли намагаюсь редагувати System PATH.
**Acceptance:**
- Спроба редагування System PATH без прав адміністратора відображає InlineAlert.
- InlineAlert містить кнопку "Run as Administrator" (ініціює UAC elevation).
- Без elevation поле System PATH доступне лише для читання.

### US-backup
**Priority:** 🔴 must
**Description:** Як користувач, я хочу, щоб перед кожним збереженням змін автоматично створювався бекап, щоб я міг відновити попередній стан у разі помилки.
**Acceptance:**
- Бекап створюється автоматично перед кожним Apply.
- Файл зберігається у `data/backups/YYYY-MM-DDTHH-MM-SS.json`.
- JSON містить поля: `timestamp`, `scope` (`user` або `system`), `entries` (масив рядків).
- Якщо кількість бекапів перевищує `maxBackups` — найстаріший видаляється.

### US-restore
**Priority:** 🔴 must
**Description:** Як користувач, я хочу переглядати список бекапів і відновлювати потрібний знімок одним кліком.
**Acceptance:**
- Вкладка "Backups" відображає список збережених знімків з датою, часом і scope.
- Кнопка "Restore" застосовує обраний знімок після підтвердження користувачем.
- Відновлення System PATH потребує прав адміністратора (аналогічно US-admin-elevation).

### US-accessibility
**Priority:** 🔴 must
**Description:** Як незрячий користувач, я хочу повністю керувати застосунком з клавіатурою і отримувати оголошення статусів через NVDA, щоб не залежати від миші або візуальних підказок.
**Acceptance:**
- Усі інтерактивні елементи мають `AccessibleName` і `AccessibleDescription`.
- При переміщенні фокусу на запис з проблемою NVDA оголошує тип проблеми текстом (наприклад "Path does not exist", "Duplicate").
- Повна навігація: Tab / Shift+Tab між елементами, стрілки у списку, Enter / Space для активації.
- Жоден сценарій не вимагає використання миші.

### US-i18n
**Priority:** 🔴 must
**Description:** Як україномовний користувач, я хочу бачити інтерфейс рідною мовою, щоб комфортно користуватись застосунком.
**Acceptance:**
- Підтримувані мови: English (en), Ukrainian (uk).
- Мова перемикається у налаштуваннях; зміни вступають в силу після перезапуску застосунку.
- Після зміни мови відображається повідомлення: "Please restart the application to apply the language change.".
- Усі рядки UI (мітки, кнопки, повідомлення, tooltips) перекладені.
- NVDA-оголошення (`AccessibleName`, `AccessibleDescription`) перекладаються відповідно до обраної мови інтерфейсу.

### US-settings
**Priority:** 🟡 should
**Description:** Як користувач, я хочу налаштувати мову, кількість бекапів та тему, щоб адаптувати застосунок під свої потреби.
**Acceptance:**
- Налаштування зберігаються у `data/settings.json`.
- При відсутності файлу використовуються дефолти: мова = system locale (en якщо не uk), maxBackups = 50, тема = system.

### US-high-contrast
**Priority:** 🔴 must
**Description:** Як користувач з вадами зору, я хочу, щоб застосунок коректно відображався у режимі High Contrast Windows, щоб я міг розрізняти елементи інтерфейсу.
**Acceptance:**
- У режимі High Contrast застосунок використовує системні кольори (не жорстко задані HEX-кольори).
- Відсутні елементи, невидимі або нечитабельні у High Contrast режимі.

---

## 6. Функціональні вимоги

### FR-view-tabs
**Priority:** 🔴 must
**Description:** Інтерфейс відображає User PATH, System PATH та Backups у трьох окремих вкладках (TabPages).
**Acceptance:**
- Вкладки: "User PATH", "System PATH", "Backups".
- При запуску активна вкладка "User PATH".
- Перемикання між вкладками підтримується з клавіатури (Ctrl+Tab / Ctrl+Shift+Tab).
- NVDA оголошує назву вкладки при перемиканні.

### FR-listview-columns
**Priority:** 🔴 must
**Description:** ListView відображає кожен запис як окремий рядок з колонками: шлях, статус.
**Acceptance:**
- Колонки: `Path` (повний рядок шляху), `Status` (текстовий статус).
- Статусні значення: `OK`, `Warning`, `Error` з відповідними іконками.

### FR-auto-diagnose
**Priority:** 🔴 must
**Description:** Автодіагностика запускається при запуску програми та після кожного Apply або скасування змін.
**Acceptance:**
- Запускається асинхронно, не блокує UI.
- Повний цикл діагностики завершується менш ніж за 1 секунду для PATH з ≤ 200 записів.

### FR-diag-duplicates
**Priority:** 🔴 must
**Description:** Діагностика виявляє дублікати записів (регістронезалежне порівняння).
**Acceptance:**
- Усі записи, що мають ідентичний шлях (case-insensitive, з нормалізацією trailing `\`), позначаються статусом `Warning: Duplicate`.
- Нормалізація використовується лише для порівняння; оригінальний рядок зберігається в реєстрі без змін.
- NVDA оголошує: "Duplicate" при фокусі на такому записі.

### FR-diag-nonexistent
**Priority:** 🔴 must
**Description:** Діагностика перевіряє існування кожного шляху на диску.
**Acceptance:**
- Шляхи, для яких `os.Stat()` повертає помилку, позначаються `Error: Path does not exist`.
- NVDA оголошує: "Path does not exist".

### FR-diag-length
**Priority:** 🔴 must
**Description:** Діагностика попереджає, якщо повна довжина рядка PATH перевищує 32767 символів.
**Acceptance:**
- Перевіряється окремо User PATH, окремо System PATH, та комбінована довжина (User + System + 1 роздільник), оскільки Windows об'єднує їх при створенні процесу.
- InlineBanner у верхній частині панелі відображає: "Warning: PATH length exceeds 32767 characters (current: N)".
- NVDA оголошує попередження при появі банера.

### FR-diag-relative
**Priority:** 🔴 must
**Description:** Діагностика виявляє відносні шляхи (`.`, `..`, шляхи без кореневого диска).
**Acceptance:**
- Відносні шляхи позначаються `Warning: Relative path (security risk)`.
- NVDA оголошує: "Relative path, security risk".

### FR-diag-empty
**Priority:** 🔴 must
**Description:** Діагностика виявляє порожні записи (пустий рядок після split по `;`).
**Acceptance:**
- Порожні записи позначаються `Warning: Empty entry`.
- NVDA оголошує: "Empty entry".

### FR-edit-f2
**Priority:** 🔴 must
**Description:** Натискання F2 або подвійний клік на запис відкриває режим inline-редагування.
**Acceptance:**
- Поле редагування з'являється у рядку запису.
- Escape скасовує редагування без змін; Enter підтверджує.
- При підтвердженні (Enter) виконується валідація: шлях не містить заборонених символів (`<`, `>`, `|`, `"`) та не є порожнім. У разі помилки — поле підсвічується і відображається текстова підказка.

### FR-add-delete
**Priority:** 🔴 must
**Description:** Кнопки "Add" і "Delete" дозволяють додавати нові та видаляти обрані записи.
**Acceptance:**
- "Add" додає порожній запис у кінець списку і відразу відкриває режим редагування.
- "Delete" видаляє обраний запис після підтвердження (confirm dialog).
- Обидві дії підтримують Undo.

### FR-reorder-keyboard
**Priority:** 🔴 must
**Description:** Користувач може змінювати порядок записів через кнопки Move Up / Move Down.
**Acceptance:**
- Move Up / Move Down доступні з клавіатури (підключені до кнопок або гарячих клавіш).
- Переміщення підтримує Undo.

### FR-reorder-dnd
**Priority:** 🟡 should
**Description:** Користувач може змінювати порядок записів через Drag & Drop.
**Acceptance:**
- Drag & Drop працює в межах однієї вкладки (User або System).
- Переміщення підтримує Undo.

### FR-undo-redo
**Priority:** 🔴 must
**Description:** Підтримка Undo / Redo для всіх операцій редагування.
**Acceptance:**
- Ctrl+Z скасовує останню операцію (стек необмежений у межах сесії).
- Ctrl+Y повторює скасовану операцію.
- Undo / Redo не застосовуються до вже збережених (Apply) змін.

### FR-apply
**Priority:** 🔴 must
**Description:** Кнопка "Apply" записує зміни у реєстр Windows і надсилає `WM_SETTINGCHANGE`.
**Acceptance:**
- Перед записом автоматично створюється бекап (FR-backup-auto).
- Перед записом порівнюється поточне значення PATH у реєстрі зі значенням, завантаженим при останньому Read/Refresh. Якщо є розбіжність — відображається діалог: "PATH was modified externally since last refresh. [Overwrite] [Refresh and discard my changes] [Cancel]".
- Після успішного Apply діагностика запускається повторно.
- У разі помилки запису (наприклад, відмова в доступі) відображається повідомлення про помилку; зміни не застосовуються.

### FR-cancel
**Priority:** 🔴 must
**Description:** Кнопка "Cancel" скасовує всі незбережені зміни та повертає список до попереднього стану.
**Acceptance:**
- Список відновлюється до стану після останнього Apply або до вихідного стану при запуску (якщо Apply не було).
- Якщо є незбережені зміни — показується діалог підтвердження "Discard changes? [Yes] [No]". Без змін — скасовується одразу.

### FR-close-confirm
**Priority:** 🔴 must
**Description:** При закритті головного вікна (Alt+F4, кнопка ×) з незбереженими змінами відображається діалог підтвердження.
**Acceptance:**
- Діалог: "You have unsaved changes. Save before closing? [Save] [Discard] [Cancel]".
- "Save" — виконує Apply, потім закриває вікно.
- "Discard" — закриває вікно без збереження.
- "Cancel" — повертає до застосунку, вікно не закривається.
- Якщо немає незбережених змін — вікно закривається одразу без діалогу.

### FR-backup-auto
**Priority:** 🔴 must
**Description:** Перед кожним Apply автоматично створюється файл бекапу.
**Acceptance:**
- Файл: `data/backups/YYYY-MM-DDTHH-MM-SS.json`.
- JSON-структура: `{ "timestamp": "ISO 8601", "scope": "user|system", "entries": ["path1", "path2", ...] }`.
- При Apply User PATH створюється бекап з `scope: "user"`. При Apply System PATH — окремий бекап з `scope: "system"`. Кожен Apply створює рівно один бекап для відповідного scope.
- Якщо директорія `data/backups/` відсутня — створюється автоматично.

### FR-backup-rotation
**Priority:** 🔴 must
**Description:** Кількість збережених бекапів обмежується налаштуванням `maxBackups`; старі видаляються автоматично.
**Acceptance:**
- Default: 50 бекапів.
- При перевищенні ліміту видаляється файл з найстарішою датою у назві.

### FR-backup-ui
**Priority:** 🔴 must
**Description:** Вкладка "Backups" відображає список знімків і дозволяє відновити будь-який з них.
**Acceptance:**
- Список показує: дата/час, scope, кількість записів у знімку.
- Кнопка "Restore" завжди відображає діалог підтвердження ("Restore this snapshot? Current PATH will be overwritten. [Yes] [No]"), незалежно від наявності незбережених змін.
- Якщо є незбережені зміни — діалог додатково попереджає: "You have unsaved changes. They will be lost."
- Відновлення System PATH потребує прав адміністратора.
- Якщо обраний бекап-файл містить невалідний JSON або відсутні обов'язкові поля — відображається повідомлення про помилку; відновлення не виконується. Пошкоджений файл позначається у списку бекапів міткою "[Corrupted]".

### FR-settings-file
**Priority:** 🟡 should
**Description:** Налаштування зберігаються у `data/settings.json` і автоматично завантажуються при запуску.
**Acceptance:**
- Файл створюється при першому запуску з дефолтними значеннями, якщо відсутній.
- Параметри: `language` (en/uk), `maxBackups` (int), `theme` (system/high-contrast).
- Змінені налаштування застосовуються без перезапуску.
- Якщо файл settings.json містить невалідний JSON або некоректні значення (наприклад, `maxBackups: -5`) — застосовуються дефолти, файл перезаписується валідною версією, у StatusBar відображається попередження: "Settings file was corrupted and has been reset to defaults.".

### FR-i18n-runtime
**Priority:** 🔴 must
**Description:** Перемикання мови зберігається у налаштуваннях і вступає в силу після перезапуску застосунку.
**Acceptance:**
- Після зміни мови у Settings відображається повідомлення: "Please restart the application to apply the language change.".
- Мова інтерфейсу змінюється після наступного запуску.
- Усі статусні повідомлення, tooltips, кнопки, заголовки і діалоги перекладені.

### FR-refresh
**Priority:** 🔴 must
**Description:** Клавіша F5 або кнопка "Refresh" перечитує PATH із реєстру Windows без перезапуску застосунку.
**Acceptance:**
- F5 або Edit → Refresh скидає поточний стан редагування (з підтвердженням якщо є незбережені зміни) і завантажує актуальні значення з реєстру.
- Після Refresh автоматично запускається діагностика.
- NVDA оголошує: "PATH refreshed".

### FR-copy-entry
**Priority:** 🟡 should
**Description:** Ctrl+C копіює текст обраного PATH-запису у системний буфер обміну.
**Acceptance:**
- Ctrl+C на обраному рядку ListView копіює повний шлях (raw, з `%VAR%` якщо є) у clipboard.
- StatusBar або toast-повідомлення підтверджує: "Copied to clipboard".
- NVDA оголошує підтвердження копіювання.

### FR-browse-folder
**Priority:** 🔴 must
**Description:** При додаванні або редагуванні запису доступна кнопка "Browse" що відкриває стандартний FolderBrowserDialog.
**Acceptance:**
- Кнопка "Browse" розміщена поруч із полем введення шляху (у режимі Add та Edit).
- Обрана папка вставляється у поле введення.
- Після вибору папки поле редагування отримує фокус.
- Кнопка доступна з клавіатури; NVDA оголошує її.

### FR-var-expansion-toggle
**Priority:** 🟡 should
**Description:** Кнопка/команда перемикає відображення між raw-значеннями (`%JAVA_HOME%\bin`) та розгорнутими (`C:\jdk21\bin`) у ListView.
**Acceptance:**
- Toggle-кнопка "Expand %VAR%" у тулбарі або View меню.
- У режимі "expanded": кожен запис показується з розгорнутими змінними; raw-рядок зберігається в реєстрі без змін.
- Якщо змінна не знайдена → запис позначається `Warning: Unknown variable %VARNAME%`.
- Зміна режиму не вважається редагуванням (не активує стан "unsaved changes").
- NVDA оголошує поточний режим при перемиканні.

### FR-menubar
**Priority:** 🔴 must
**Description:** Головне меню (MenuBar) надає доступ до всіх команд застосунку через Alt-навігацію, що критично для NVDA-користувачів.
**Acceptance:**
- Структура меню: `File` | `Edit` | `View` | `Tools` | `Help`.
- `File`: Save (Apply), Restore Backup, Exit.
- `Edit`: Add Entry, Delete Entry, Move Up, Move Down, Undo, Redo, Fix Issues.
- `View`: Filter (All/Issues/Errors/Warnings), Tree View, Search (Ctrl+F).
- `Tools`: Settings, Open Backups Folder.
- `Help`: About, Keyboard Shortcuts.
- Усі пункти меню мають hotkey (підкреслена буква + Alt) і відображаються з keyboard shortcuts (наприклад `Ctrl+Z`).
- NVDA повністю читає MenuBar: назви, стан (enabled/disabled), shortcuts.

### FR-statusbar
**Priority:** 🟡 should
**Description:** StatusBar у нижній частині вікна відображає зведення стану PATH без сканування ListView.
**Acceptance:**
- Вміст: `User PATH: N entries (M issues) | System PATH: N entries (M issues) | Total length: N chars [⚠ if > 32767]`.
- Оновлюється після кожної діагностики та після кожного Apply.
- Якщо загальна довжина PATH > 32767 — секція "Total length" виділяється і містить текст "⚠ Exceeds limit".

### FR-search
**Priority:** 🟡 should
**Description:** Search bar (Ctrl+F) дозволяє фільтрувати ListView за підрядком шляху в реальному часі.
**Acceptance:**
- Ctrl+F переміщує фокус у поле пошуку над ListView.
- Фільтрація відбувається при кожній зміні тексту (live, без кнопки "Search").
- Пошук регістронезалежний (case-insensitive), по тексту шляху.
- ESC очищає текст пошуку і повертає фокус у ListView.
- Пошук і Filter bar (All/Issues/Errors/Warnings) діють одночасно (AND-логіка).
- Індекси записів у результатах відповідають оригінальним позиціям у PATH.
- При активному пошуку відображається лічильник: "N of M entries".
- NVDA оголошує кількість результатів після паузи введення (debounce ~300 мс).

### FR-tree-browser
**Priority:** 🟡 should
**Description:** Кнопка "Tree View" (або клавіша) відкриває модальний діалог, що відображає всі записи PATH у вигляді дерева файлової системи. Вибір вузлів-листів закриває діалог і переміщує фокус на відповідний рядок у ListView.
**Acceptance:**
- Діалог відкривається кнопкою в тулбарі або гарячою клавішею (наприклад Alt+T).
- Дерево будується шляхом розбиття кожного PATH-запису на компоненти шляху (`C:\`, `Windows\`, `System32`); проміжні вузли — суто навігаційні.
- Листові вузли (terminal nodes) = повні PATH-записи. Тільки вони є "вибираємими".
- Розгортання змінних середовища (`%VAR%`): дерево будується по розгорнутих шляхах; у деталях вузла-листа показується оригінальний рядок з `%VAR%`.
- Клавіші: стрілки для навігації, Enter / кнопка "Go To" для вибору вузла, Escape для скасування.
- **Enter на будь-якому вузлі** (листовому або проміжному): вставляє повний шлях вузла у Search bar головного вікна та закриває діалог. Фокус переміщується у Search bar з виділеним текстом.
- **Виняток**: якщо вузол є листом (повний PATH-запис) — окрім заповнення Search bar, ListView також прокручується до відповідного запису і виділяє його.
- Вузол-лист що має issue отримує візуальну та текстову мітку статусу (наприклад `[Error]`).
- Діалог не дозволяє редагувати записи — лише навігація.
- NVDA: кожен TreeItem має `AccessibleName` = повний шлях вузла; статус issue оголошується.

### FR-filter-bar
**Priority:** 🟡 should
**Description:** Над ListView відображається панель фільтрів, що дозволяє показувати лише записи з певним статусом.
**Acceptance:**
- Filter bar містить radio/toggle-кнопки: "All", "Issues only", "Errors", "Warnings".
- Default: "All".
- Фільтрація змінює відображення ListView без зміни фактичного порядку записів у PATH.
- Індекси (#) у відфільтрованому списку відповідають оригінальним позиціям у PATH (не перенумеровуються).
- Кожна кнопка доступна з клавіатури; NVDA оголошує кількість записів у результаті фільтрації (наприклад "3 items").
- При активному фільтрі відображається нагадування: "Filtered view — N of M entries shown".

### FR-fix-issues
**Priority:** 🟡 should
**Description:** Кнопка "Fix Issues" відкриває preview-діалог зі списком усіх знайдених проблем, де користувач обирає через чекбокси які з них виправити, після чого застосовує зміни одним кліком.
**Acceptance:**
- Кнопка "Fix Issues" відображається в тулбарі і активна лише якщо є хоча б один issue.
- Діалог містить ListView: кожен рядок = одна знайдена проблема з чекбоксом, типом issue, шляхом та запропонованою дією (видалити / пропустити).
- За замовчуванням увімкнені: дублікати, порожні записи.
- За замовчуванням вимкнені: неіснуючі шляхи з мережевих або змінних дисків (`\\`, `%`-шляхи, диски без DriveType=Fixed).
- Неіснуючі шляхи на локальних фіксованих дисках увімкнені за замовчуванням.
- Кнопка "Apply Selected" застосовує всі відмічені виправлення як одну операцію, яку можна скасувати через Undo (Ctrl+Z).
- Після виправлень автоматично запускається діагностика.
- Усі елементи діалогу доступні з клавіатури; NVDA оголошує тип і статус кожного рядка.

### FR-uac-elevation
**Priority:** 🔴 must
**Description:** Спроба редагувати System PATH без прав адміністратора ініціює UAC elevation або відображає InlineAlert.
**Acceptance:**
- Якщо поточний процес не має прав адміністратора — System PATH відображається як read-only.
- InlineAlert містить текст про необхідність адміністративних прав і кнопку "Run as Administrator".
- Кнопка перезапускає застосунок з `ShellExecute("runas", ...)`.
- Перед перезапуском з elevation, якщо є незбережені зміни, відображається діалог: "Unsaved changes will be lost. Continue? [Yes] [No]". Якщо "No" — elevation скасовується.

---

## 7. Нефункціональні вимоги

### NFR-portable
**Priority:** 🔴 must
**Description:** Застосунок розповсюджується як єдиний `.exe` файл без інсталятора.
**Acceptance:**
- Усі ресурси (іконки, переклади) вбудовані в EXE
- Запуск можливий без попередньої установки будь-яких залежностей (VC++ Runtime тощо).

### NFR-no-registry-writes
**Priority:** 🔴 must
**Description:** Застосунок не записує нічого до реєстру Windows, AppData, `%TEMP%` або будь-яких системних розташувань (крім цільових ключів PATH при Apply).
**Acceptance:**
- Єдині записи до реєстру — ключі `HKCU\Environment` (User PATH) і `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` (System PATH) при Apply.
- Перевірка: Process Monitor не фіксує запис до `HKCU\Software`, AppData або `%TEMP%` при звичайній роботі.

### NFR-startup-time
**Priority:** 🔴 must
**Description:** Час запуску не перевищує 2 секунд на SSD.
**Acceptance:**
- Вимірюється від запуску процесу до появи головного вікна з заповненим списком PATH.
- Тест: Windows 10 21H2 на SSD NVMe, PATH з 50 записами.

### NFR-exe-size
**Priority:** 🟡 should
**Description:** Розмір EXE-файлу не перевищує 20 MB.
**Acceptance:**
- `PathMaster.exe` ≤ 20 MB після компіляції з оптимізаціями

### NFR-compatibility
**Priority:** 🔴 must
**Description:** Застосунок працює на Windows 10 21H2+ та Windows 11 (x64).
**Acceptance:**
- Функціональні тести пройдені на Windows 10 21H2 і Windows 11 23H2.
- 32-bit Windows не підтримується.

### NFR-accessibility-wcag
**Priority:** 🔴 must
**Description:** Інтерфейс відповідає стандарту WCAG 2.1 AA щодо контрастності та клавіатурної навігації.
**Acceptance:**
- Контрастне відношення тексту до фону ≥ 4.5:1 у всіх темах (крім High Contrast — там системні кольори).
- Усі інтерактивні елементи досяжні з клавіатури без пастки фокусу.

### NFR-no-color-only
**Priority:** 🔴 must
**Description:** Колір ніколи не є єдиним індикатором статусу або дії.
**Acceptance:**
- Статусні повідомлення та іконки завжди супроводжуються текстовою міткою.

### NFR-window-sizing
**Priority:** 🔴 must
**Description:** Головне вікно resizable з мінімальним розміром.
**Acceptance:**
- Мінімальний розмір вікна: 800×600 px.
- ListView та панелі масштабуються при зміні розміру.
- Підтримується Maximize (Win+Up).

### NFR-logging
**Priority:** 🟡 should
**Description:** Застосунок записує діагностичний лог для пошуку проблем.
**Acceptance:**
- Лог-файл: `data/pathmaster.log` (ротація за розміром, max 5 MB).
- Лог містить: запуск, Apply, Restore, помилки реєстру, UAC-події.
- Телеметрія та мережеві з'єднання відсутні.

---

## 8. Технічні обмеження

### TC-file-structure
**Priority:** 🔴 must
**Description:** Файлова структура застосунку фіксована.
**Acceptance:**
- `PathMaster.exe` — виконуваний файл.
- `data/settings.json` — налаштування.
- `data/backups/*.json` — бекапи.
- `data/pathmaster.log` — діагностичний лог (якщо NFR-logging реалізовано).
- Застосунок не створює файлів поза цією структурою.

### TC-registry-keys
**Priority:** 🔴 must
**Description:** Читання та запис PATH відбувається через конкретні ключі реєстру.
**Acceptance:**
- User PATH: `HKCU\Environment`, значення `Path` (REG_EXPAND_SZ).
- System PATH: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`, значення `Path` (REG_EXPAND_SZ).
- Після запису надсилається `WM_SETTINGCHANGE` з параметром `"Environment"` для сповіщення запущених процесів.

### TC-wm-settingchange
**Priority:** 🔴 must
**Description:** Після успішного Apply застосунок надсилає broadcast `WM_SETTINGCHANGE`.
**Acceptance:**
- Виклик: `SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment", SMTO_ABORTIFHUNG, 5000, &result)`.
- Timeout: 5000 мс; помилка timeout не вважається критичною і логується.

---

## 10. Поза scope

### OS-other-env-vars
**Priority:** 🔴 must
**Description:** Редагування будь-яких змінних середовища, крім `PATH`.
**Acceptance:**
- Застосунок читає та редагує виключно значення `Path`; інші змінні не відображаються і не змінюються.

### OS-sync
**Priority:** 🔴 must
**Description:** Синхронізація PATH між різними машинами або хмарне зберігання.
**Acceptance:**
- Відсутні будь-які механізми мережевої передачі або хмарного зберігання.

### OS-plugins
**Priority:** 🔴 must
**Description:** Система плагінів або розширень.
**Acceptance:**
- Відсутні API, хуки або механізми для завантаження стороннього коду.

### OS-web-cli
**Priority:** 🔴 must
**Description:** Web-інтерфейс або CLI-версія застосунку.
**Acceptance:**
- Застосунок існує виключно як Win32 GUI EXE.

### OS-auto-update
**Priority:** 🔴 must
**Description:** Автооновлення та перевірка нових версій.
**Acceptance:**
- Автооновлення та перевірка нових версій відсутні у MVP. Користувач завантажує нові версії вручну.

---

## Додаткові вимоги

- можливість встановити та оновити за допомогою scoop
- можливість встановити та оновити за допомогою winget
