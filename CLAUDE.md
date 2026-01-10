# Claude Skills

## Documentation Standards

### Description
Guidelines for writing documentation and docstrings in this project.

### Instructions

When writing documentation or docstrings for this project, follow these standards:

#### Documentation Format

**All standalone documentation files must be written in AsciiDoc format (.adoc files).**

For inline code documentation (Rust docstrings), use the standard Rust documentation format with `///` or `//!` comments.

#### Style Guide

The documentation style is based on the [Readability Guidelines](https://readabilityguidelines.co.uk) by Content Design London, chosen for its focus on readability and open community approach.

#### Language and Spelling

- Use Australian English (en_AU) spelling
  - "behaviour" not "behavior"
  - "cancelled" not "canceled"

#### Writing Principles

**Keep it simple:**
- Use short, simple words where possible
- Use formal language
- Avoid long sentences
- Prefer short sentences over complex combined sentences
- Each sentence should convey one idea
- Ensure all sentences are complete and grammatically correct
- Use articles (the, a, an) appropriately before nouns

**Do not use:**
- Slang (for example, "there you go")
- Jargon (for example, "leverage", "streamline")
- Ambiguous contractions (for example, "there'd", "it'll")
- Latin terms (for example, "i.e.", "e.g.", "etc.", "vs.", "via")
  - Write "for example" and "that is" in full
- Metaphors (for example, "cherry picking", "nutshell")
- Complex or specialist terms without explanation (for example, "chrome" for UI elements, "upstream")

For guidelines on specialist terms, see: [Readability Guidelines - Specialist terms](https://readabilityguidelines.co.uk/clear-language/specialist-terms/)

**Voice and perspective:**
- Use active voice, not passive voice
- Write in second person perspective when addressing the user
- Avoid pronouns where possible (including "we", "you", "I", "their")

**Avoid adverbs:**
- Do not use adverbs such as "very" and "usually"
- Avoid weakening adverbs (like `recently`, `immediately`, `early`)
- These words make writing less direct and weaken meaning

**Avoid contractions:**
- Do not use contractions such as "can't", "don't", "they're", "could've"

#### Proper Names

When referring to development languages, use proper names or industry conventions:
- "CSS" not "css"
- "jQuery" not "Jquery"
- "React" not "React-js"

When referring to programs, use proper names:
- "Microsoft Internet Explorer" not "IE"
- "Apache Tomcat" not "Tomcat"

In code blocks, use standard syntax as appropriate.

#### Titles and Headings

- Make titles and headings descriptive but concise
- Use sentence-case capitalization (not Title Case)
- Sentence case is more comfortable to read in technical documentation

#### Abbreviations, Acronyms, and Initialisms

- Do not use points or spaces
- Write "for example" and "that is" in full (not "eg" or "ie")
- If an acronym is better understood than the full text, use the acronym
- Use all capital letters for initialisms (for example, API, HTML)
- Start with a capital letter for acronyms (for example, Nasa)
- Capitalize single letters in expressions
- Provide full text explanations on first use
- Consider providing a full explanation each time for clarity

#### Hyphens

Limit use of hyphens:
- Only use a hyphen if the word or phrase is confusing without it
- Ensure hyphen usage is up to date
- Be consistent with hyphen usage

#### Links and Cross-References

When adding links or cross-references:
- Make link text meaningful
- Avoid mid-sentence links
- Match the destination content
- Use sentence case

#### Images and Icons

- Avoid adding images to documentation when possible (they quickly become outdated)
- Use descriptions or demonstrations instead
- When images are necessary, reuse existing images if possible
- Provide brief alternative text descriptions for accessibility

#### User Interface Documentation

When describing user interface elements and interactions:
- Follow the [Microsoft Style Guide](https://docs.microsoft.com/en-us/style-guide) for UI descriptions
- Be consistent with terminology for UI interactions (click, select, tap, swipe)
- Use proper formatting for UI element names

#### Admonitions (Notes, Warnings, etc.)

Avoid overusing admonitions to prevent "Admonition Fatigue".

Use four levels of admonition (in order of severity):

**NOTE:** Use for additional, indirectly related information. Do not use where it is possible to reword or rewrite the content to incorporate the information.

**IMPORTANT:** Use when ignoring the notice may or will lead to unexpected behaviour.

**CAUTION:** Use when ignoring the notice may lead to:
- A significant increase in the risk of a security breach
- Creation of a security vulnerability
- Information loss
- System failure
- Worse outcomes than those listed here

**WARNING:** Use when ignoring the notice will lead to:
- A significant increase in the risk of a security breach
- Creation of a security vulnerability
- Information loss
- System failure
- Worse outcomes than those listed here

#### Rust Docstring Format

For Rust code documentation:
- Use `///` for item documentation
- Use `//!` for module/crate-level documentation
- Include sections: Description, Arguments, Returns, Examples, Errors, Panics (as applicable)
- Provide code examples in docstrings where helpful
- Follow the existing documentation style in the project

#### Writing Tips

- Focus on how users read content (they scan rather than read every word)
- Structure content logically with clear headings
- Keep paragraphs short and focused on a single idea
- Use lists to break up dense information
- Front-load important information

#### Code Style Decisions

**Rust Code Standards:**

- **Imports**: Use `crate::` prefix for internal module imports (not relative imports)
  - Good: `use crate::common::{App, NotificationType};`
  - Bad: `use common::{App, NotificationType};`

- **Error Handling**:
  - Use `Result` types for recoverable errors
  - Use `expect()` for errors that should not occur in normal operation
  - Provide descriptive error messages in `expect()` calls

- **Function Organization**:
  - Extract pure functions for testability
  - Keep I/O operations separate from business logic
  - Public functions for testable logic, private for internal helpers

- **Testing**:
  - Comprehensive unit tests for all public functions
  - Test edge cases (zero, one, many)
  - Test error conditions where applicable
  - Use descriptive test names that explain what is being tested

- **Documentation**:
  - All public functions must have doc comments
  - Include Arguments, Returns, Examples, Errors, and Panics sections as applicable
  - Module-level documentation using `//!` comments

### Resources

- [Content Design London: Readability Guidelines](https://readabilityguidelines.co.uk/)
- [Microsoft Style Guide](https://docs.microsoft.com/en-us/style-guide)
- [Plain Language Guidelines](https://plainlanguage.gov/resources/articles/dash-writing-tips/)
- [Australian Government Style Manual - Latin shortened forms](https://www.stylemanual.gov.au/format-writing-and-structure/clear-language-and-writing-style/plain-english-and-word-choice/latin-shortened-forms)

### Examples

Good example:
```rust
/// Formats an update message based on the number of available updates.
///
/// # Arguments
///
/// * `count` - The number of available updates
///
/// # Returns
///
/// A formatted message string describing the updates and how to install them.
/// Uses singular form for 1 update, plural for multiple updates.
pub fn format_update_message(count: usize) -> String {
    // implementation
}
```

Bad example:
```rust
/// Formats the msg (i.e., creates a string w/ update info)
/// We leverage this function to streamline the UX
pub fn format_update_message(count: usize) -> String {
    // implementation
}
```
