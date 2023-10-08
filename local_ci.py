import os
import yaml
import click


def confirm_app_version(app: str):
    current_version = ""
    current_app = ""
    data = None
    with open(f"manifests{os.sep}{app}", "r") as file:
        data = list(yaml.safe_load_all(file))
        for d in data:
            if "Deployment" == d["kind"]:
                current_image, current_version = d["spec"]["template"]["spec"]["containers"][0]["image"].split(":")
                current_app = app.split('.')[0]
                if click.confirm(f"是否需要修改{current_app}版本, 当前{current_version}"):
                    new_version = click.prompt(f"输入{current_app}新版本号，当前{current_version}")
                    d["spec"]["template"]["spec"]["containers"][0]["image"] = f"{current_image}:{new_version}"
    with open(f"manifests{os.sep}{app}", "w") as file:
        yaml.dump_all(data, file)


if __name__ == "__main__":
    yamls = os.listdir("manifests")
    for y in yamls:
        confirm_app_version(y)
