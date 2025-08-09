# Node.js LLM Client

This is an unofficial project solely intended for personal learning and research. This repository contains a Node.js command line application that can be used to perform inferences on data from various sources such as files, directories, PDFs, images, web pages, and Git repositories using the GPT-4 API. For comparison, the application integrates with multiple AI models including OpenAI's GPT models, Google's Generative AI, Anthropic's Claude, and Mistral AI for processing natural language and generating responses. In order to use the application you must have an API key for the various AI models that the application supports. Please see the instructions below for more details.

## Features

- **File Processing**: Extract text from plain text files.
- **PDF Processing**: Extract text and optionally images from PDF documents.
- **Image Processing**: Send image URLs directly to the AI model.
- **Directory Processing**: Recursively process and extract data from directories.
- **Git Repository Processing**: Clone Git repositories and process their contents.
- **Web Page Processing**: Scrape text and optionally images from web pages.
- **Excel Processing**: Convert Excel files to CSV and process the data.
- **Data Display**: Option to display extracted data before sending it to the AI model.
- **Cost Estimation**: Estimate the token and cost implications of processing images and text data.
- **Model Selection**: Choose between different AI models for processing the data.
- **Termux Support**: Enhanced compatibility for running the application within the Termux environment on Android devices.

## Installation

To install the necessary dependencies, run the following command:

```bash
npm install
```

### Termux Support

For users running this application within the Termux environment on Android devices, there is a specific requirement for the `sharp` library due to its native dependencies. To ensure compatibility, use the following commands:

```bash
npm install --force @img/sharp-wasm32
vi package.json and remove puppeteer from the dependencies
```

This command forces the installation of a WebAssembly (WASM) version of the `sharp` library, which is compatible with the Termux environment. It is also necessary to remove the puppeteer dependency from the `package.json` file, as it is not compatible with Termux.

```bash
git reset --hard origin/develop
```

This command updates the local repository with the latest changes from the remote repository, ignoring any local changes. This is required since the package.json file is modified during the installation process.

## Usage

To start the application, run:

```bash
node --no-warnings Main.js
```

Follow the prompts to input commands and paths to data sources. The application will process the data and interact with the AI model to generate responses.

## Commands

The first letter can be used as a shortcut for each command. Paths can be relative, absolute, or URLs.

- `file`: Process a text file.
- `pdf`: Process a PDF file. If a local path without the `.pdf` extension is
  provided, the extension will be checked automatically.
- `xlsx`: Process an Excel file.
- `image`: Process an image URL.
- `dir`: Process a directory.
- `git`: Process a Git repository.
- `web`: Process a web page.
- `save`: Save the current chat session.
- `exit`: Exit the application.

## Models

- **OpenAI**: https://platform.openai.com/docs/quickstart
- **Google**: https://ai.google.dev/docs
- **Anthropic**: https://docs.anthropic.com/claude/reference/getting-started-with-the-api
- **Mistral AI**: https://docs.mistral.ai/#api-access

### Message Templates

Message formatting rules for each provider are defined in `messageTemplates.js`.
Api clients automatically load the correct template, so you can customize how
messages are sent by editing that file.

## Configuration

Copy `.env.example` to `.env` and fill in your API keys. The application uses the [dotenv](https://www.npmjs.com/package/dotenv) package to load variables automatically:

- `OPENAI_API_KEY`: Your OpenAI API key.
- `GOOGLE_AI_API_KEY`: Your Google Generative AI API key.
- `ANTHROPIC_API_KEY`: Your Anthropic API key.
- `MISTRAL_API_KEY`: Your Mistral AI API key.
- `PERPLEXITY_API_KEY`: Your Perplexity AI key.
- `OLLAMA_API_KEY`: Your Ollama API key.
- `OLLAMA_API_URL`: Base URL for your Ollama server (default `http://127.0.0.1:11434`).

## App Data

The application stores data in specific directories within the user's home directory or, for Termux users, in the download directory. Here's a quick overview:

- **Chat Transcripts**: Saved in `~/.chatgpt-client/chats`, each session is stored in a Markdown file named with the session's start timestamp.

- **Processed Files**: Files processed by the application, such as converted or extracted files, are stored in `~/.chatgpt-client/files`.

- **Temporary Data**: Used for intermediate processing steps, temporary data is stored in `~/.chatgpt-client/temp` and is managed automatically by the application.

## Known Issues

- Image processing is not fully implemented for non-OpenAI models.
- Image extraction from PDFs is spotty.
- Error handling is extremely limited.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or create issues for bugs and feature requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Featured In

[What I learned using large language models with large context windows](https://chadlangston.medium.com/9d408da8f935)
<div> 
    <a href="https://chadlangston.medium.com/9d408da8f935">
      <img alt="What I learned using large language models with large context windows" width="50%" height="50%" src="https://images.squarespace-cdn.com/content/v1/65984fc5675bb439862b1882/34379b71-48d2-41dc-a3ec-6c64ecbc0f76/neeqolah-creative-works-u8Kyb3ZV_WI-unsplash+copy.jpg?format=2500w" title="What I learned using large language models with large context windows"/>
    </a>
</div>
