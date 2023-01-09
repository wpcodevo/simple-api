use crate::{
    model::{QueryOptions, Todo, UpdateTodoSchema},
    response::{GenericResponse, SingleTodoResponse, TodoData, TodoListResponse},
    WebResult, DB,
};
use chrono::prelude::*;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, reply::with_status, Reply};

pub async fn health_checker_handler() -> WebResult<impl Reply> {
    const MESSAGE: &str = "Build Simple CRUD API with Rust";

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };
    Ok(json(response_json))
}

pub async fn todos_list_handler(opts: QueryOptions, db: DB) -> WebResult<impl Reply> {
    let todos = db.lock().await;

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let todos: Vec<Todo> = todos.clone().into_iter().skip(offset).take(limit).collect();

    let json_response = TodoListResponse {
        status: "success".to_string(),
        results: todos.len(),
        todos,
    };
    Ok(json(&json_response))
}

pub async fn create_todo_handler(mut body: Todo, db: DB) -> WebResult<impl Reply> {
    let mut vec = db.lock().await;

    for todo in vec.iter() {
        if todo.title == body.title {
            let error_response = GenericResponse {
                status: "fail".to_string(),
                message: format!("Todo with title: '{}' already exists", todo.title),
            };
            return Ok(with_status(json(&error_response), StatusCode::CONFLICT));
        }
    }

    let uuid_id = Uuid::new_v4();
    let datetime = Utc::now();

    body.id = Some(uuid_id.to_string());
    body.completed = Some(false);
    body.createdAt = Some(datetime);
    body.updatedAt = Some(datetime);

    let todo = body.to_owned();

    vec.push(body);

    let json_response = SingleTodoResponse {
        status: "success".to_string(),
        data: TodoData { todo },
    };

    Ok(with_status(json(&json_response), StatusCode::CREATED))
}

pub async fn get_todo_handler(id: String, db: DB) -> WebResult<impl Reply> {
    let vec = db.lock().await;

    for todo in vec.iter() {
        if todo.id == Some(id.to_owned()) {
            let json_response = SingleTodoResponse {
                status: "success".to_string(),
                data: TodoData { todo: todo.clone() },
            };

            return Ok(with_status(json(&json_response), StatusCode::OK));
        }
    }

    let error_response = GenericResponse {
        status: "fail".to_string(),
        message: format!("Todo with ID: {} not found", id),
    };
    return Ok(with_status(json(&error_response), StatusCode::NOT_FOUND));
}

pub async fn edit_todo_handler(
    id: String,
    body: UpdateTodoSchema,
    db: DB,
) -> WebResult<impl Reply> {
    let mut vec = db.lock().await;

    for todo in vec.iter_mut() {
        if todo.id == Some(id.clone()) {
            let datetime = Utc::now();
            let title = body.title.to_owned().unwrap_or(todo.title.to_owned());
            let content = body.content.to_owned().unwrap_or(todo.content.to_owned());
            let payload = Todo {
                id: todo.id.to_owned(),
                title: if !title.is_empty() {
                    title
                } else {
                    todo.title.to_owned()
                },
                content: if !content.is_empty() {
                    content
                } else {
                    todo.content.to_owned()
                },
                completed: if body.completed.is_some() {
                    body.completed
                } else {
                    todo.completed
                },
                createdAt: todo.createdAt,
                updatedAt: Some(datetime),
            };
            *todo = payload;

            let json_response = SingleTodoResponse {
                status: "success".to_string(),
                data: TodoData { todo: todo.clone() },
            };
            return Ok(with_status(json(&json_response), StatusCode::OK));
        }
    }

    let error_response = GenericResponse {
        status: "fail".to_string(),
        message: format!("Todo with ID: {} not found", id),
    };

    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
}

pub async fn delete_todo_handler(id: String, db: DB) -> WebResult<impl Reply> {
    let mut vec = db.lock().await;

    for todo in vec.iter_mut() {
        if todo.id == Some(id.clone()) {
            vec.retain(|todo| todo.id != Some(id.to_owned()));
            return Ok(with_status(json(&""), StatusCode::NO_CONTENT));
        }
    }

    let error_response = GenericResponse {
        status: "fail".to_string(),
        message: format!("Todo with ID: {} not found", id),
    };
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
}
